use std::str::FromStr;

use rust_decimal::Decimal;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::{
    error::DbError,
    infra::csv::ib_report_parser::view::{AccountInfo, StatementInfo},
    portfolio::summary::{
        PortfolioSummary, PortfolioTotals, PositionSummary, StoredPortfolioSummary,
        StoredReportListItem,
    },
};

pub struct PortfolioSummaryRepository;

impl PortfolioSummaryRepository {
    pub async fn insert(
        tx: &mut Transaction<'_, Sqlite>,
        summary: &PortfolioSummary,
    ) -> Result<i64, DbError> {
        let market_value = summary.totals.market_value.to_string();
        let cost_basis = summary.totals.cost_basis.to_string();
        let unrealized_pl = summary.totals.unrealized_pl.to_string();
        let unrealized_pl_percent = summary.totals.unrealized_pl_percent.to_string();
        let total_positions = summary.totals.total_positions as i64;

        let result = sqlx::query!(
            r#"
            INSERT INTO portfolio_summaries (
                account_type,
                customer_type,
                account_capabilities,
                base_currency,
                account_name,
                statement_period,
                generated_at,
                statement_title,
                broker_name,
                broker_address,
                market_value,
                cost_basis,
                unrealized_pl,
                unrealized_pl_percent,
                total_positions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            summary.account_info.account_type,
            summary.account_info.customer_type,
            summary.account_info.account_capabilities,
            summary.account_info.base_currency,
            summary.account_info.name,
            summary.statement.period,
            summary.statement.generated_at,
            summary.statement.title,
            summary.statement.broker_name,
            summary.statement.broker_address,
            market_value,
            cost_basis,
            unrealized_pl,
            unrealized_pl_percent,
            total_positions,
        )
        .execute(&mut **tx)
        .await
        .map_err(DbError::Query)?;

        let summary_id = result.last_insert_rowid();

        for position in &summary.positions {
            let quantity = position.quantity.to_string();
            let price = position.price.to_string();
            let market_value = position.market_value.to_string();
            let allocation_percent = position.allocation_percent.to_string();
            let unrealized_pl = position.unrealized_pl.to_string();
            let roi_percent = position.roi_percent.to_string();

            sqlx::query!(
                r#"
                INSERT INTO portfolio_positions (
                    summary_id,
                    symbol,
                    quantity,
                    price,
                    market_value,
                    allocation_percent,
                    unrealized_pl,
                    roi_percent
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                summary_id,
                position.symbol,
                quantity,
                price,
                market_value,
                allocation_percent,
                unrealized_pl,
                roi_percent
            )
            .execute(&mut **tx)
            .await
            .map_err(DbError::Query)?;
        }

        Ok(summary_id)
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<PortfolioSummary>, DbError> {
        let summary_row = sqlx::query!(
            r#"
            SELECT
                account_type,
                customer_type,
                account_capabilities,
                base_currency,
                account_name,
                statement_period,
                generated_at,
                statement_title,
                broker_name,
                broker_address,
                market_value,
                cost_basis,
                unrealized_pl,
                unrealized_pl_percent,
                total_positions
            FROM portfolio_summaries
            WHERE id = ?
            "#,
            id,
        )
        .fetch_optional(pool)
        .await
        .map_err(DbError::Query)?;

        let Some(summary_row) = summary_row else {
            return Ok(None);
        };

        let position_rows = sqlx::query!(
            r#"
            SELECT
                symbol,
                quantity,
                price,
                market_value,
                allocation_percent,
                unrealized_pl,
                roi_percent
            FROM portfolio_positions
            WHERE summary_id = ?
            ORDER BY id
            "#,
            id,
        )
        .fetch_all(pool)
        .await
        .map_err(DbError::Query)?;

        let total_positions = usize::try_from(summary_row.total_positions).map_err(|_| {
            DbError::InvalidData(format!(
                "total_positions must be non-negative, got {}",
                summary_row.total_positions
            ))
        })?;

        let positions = position_rows
            .into_iter()
            .map(|row| {
                Ok(PositionSummary {
                    symbol: row.symbol,
                    quantity: parse_decimal(&row.quantity, "portfolio_positions.quantity")?,
                    price: parse_decimal(&row.price, "portfolio_positions.price")?,
                    market_value: parse_decimal(
                        &row.market_value,
                        "portfolio_positions.market_value",
                    )?,
                    allocation_percent: parse_decimal(
                        &row.allocation_percent,
                        "portfolio_positions.allocation_percent",
                    )?,
                    unrealized_pl: parse_decimal(
                        &row.unrealized_pl,
                        "portfolio_positions.unrealized_pl",
                    )?,
                    roi_percent: parse_decimal(
                        &row.roi_percent,
                        "portfolio_positions.roi_percent",
                    )?,
                })
            })
            .collect::<Result<Vec<_>, DbError>>()?;

        let statement = StatementInfo {
            title: summary_row.statement_title,
            broker_name: summary_row.broker_name,
            broker_address: summary_row.broker_address,
            period: summary_row.statement_period,
            generated_at: summary_row.generated_at,
        };
        let account_info = AccountInfo {
            account_capabilities: summary_row.account_capabilities,
            account_type: summary_row.account_type,
            base_currency: summary_row.base_currency,
            customer_type: summary_row.customer_type,
            name: summary_row.account_name,
        };
        let totals = PortfolioTotals {
            market_value: parse_decimal(
                &summary_row.market_value,
                "portfolio_summaries.market_value",
            )?,
            cost_basis: parse_decimal(&summary_row.cost_basis, "portfolio_summaries.cost_basis")?,
            unrealized_pl: parse_decimal(
                &summary_row.unrealized_pl,
                "portfolio_summaries.unrealized_pl",
            )?,
            unrealized_pl_percent: parse_decimal(
                &summary_row.unrealized_pl_percent,
                "portfolio_summaries.unrealized_pl_percent",
            )?,
            total_positions,
        };

        Ok(Some(PortfolioSummary {
            statement,
            account_info,
            totals,
            positions,
        }))
    }

    pub async fn find_latest(pool: &SqlitePool) -> Result<Option<StoredPortfolioSummary>, DbError> {
        let latest_id = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM portfolio_summaries
            ORDER BY id DESC
            LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await
        .map_err(DbError::Query)?;

        let Some(latest_id) = latest_id else {
            return Ok(None);
        };

        let portfolio = Self::find_by_id(pool, latest_id).await;

        let Some(portfolio) = portfolio? else {
            return Ok(None);
        };

        Ok(Some(StoredPortfolioSummary {
            id: latest_id,
            portfolio,
        }))
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<StoredReportListItem>, DbError> {
        let list_rows = sqlx::query!(
            r#"
                SELECT
                    id,
                    account_name,
                    statement_period,
                    created_at,
                    total_positions
                FROM portfolio_summaries
                ORDER BY id DESC;
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(DbError::Query)?;

        list_rows
            .into_iter()
            .map(|row| {
                let total_positions = usize::try_from(row.total_positions).map_err(|_| {
                    DbError::InvalidData(format!(
                        "Invalid total_positions: {}",
                        row.total_positions
                    ))
                })?;
                Ok(StoredReportListItem {
                    id: row.id,
                    account_name: row.account_name,
                    statement_period: row.statement_period,
                    created_at: row.created_at,
                    total_positions,
                })
            })
            .collect()
    }

    pub async fn delete_by_id(pool: &SqlitePool, id: i64) -> Result<bool, DbError> {
        let result = sqlx::query!(
            r#"
                DELETE FROM portfolio_summaries WHERE id = ?;
            "#,
            id
        )
        .execute(pool)
        .await
        .map_err(DbError::Query)?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_all(pool: &SqlitePool) -> Result<u64, DbError> {
        let result = sqlx::query!(
            r#"
               DELETE FROM portfolio_summaries;
            "#
        )
        .execute(pool)
        .await
        .map_err(DbError::Query)?;

        Ok(result.rows_affected())
    }
}

fn parse_decimal(value: &str, field: &'static str) -> Result<Decimal, DbError> {
    Decimal::from_str(value).map_err(|source| DbError::InvalidDecimal { field, source })
}

#[cfg(test)]
mod tests {
    use sqlx::{
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
        SqlitePool,
    };

    use super::*;

    async fn setup_test_db() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("valid in-memory sqlite url")
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Memory);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect test db");

        sqlx::migrate!("./src/infra/persistence/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        pool
    }

    fn sample_portfolio_summary() -> PortfolioSummary {
        PortfolioSummary {
            statement: StatementInfo {
                title: Some("Activity Statement".to_string()),
                broker_name: Some("Interactive Brokers".to_string()),
                broker_address: None,
                period: Some("2026-06-01".to_string()),
                generated_at: Some("2026-06-01T10:00:00Z".to_string()),
            },
            account_info: AccountInfo {
                account_capabilities: Some("Margin".to_string()),
                account_type: Some("Individual".to_string()),
                base_currency: Some("USD".to_string()),
                customer_type: Some("Individual".to_string()),
                name: Some("Test Account".to_string()),
            },
            totals: PortfolioTotals {
                market_value: rust_decimal::dec!(1000.50),
                cost_basis: rust_decimal::dec!(900.25),
                unrealized_pl: rust_decimal::dec!(100.25),
                unrealized_pl_percent: rust_decimal::dec!(11.14),
                total_positions: 1,
            },
            positions: vec![PositionSummary {
                symbol: "VWCE".to_string(),
                quantity: rust_decimal::dec!(10),
                price: rust_decimal::dec!(100.05),
                market_value: rust_decimal::dec!(1000.50),
                allocation_percent: rust_decimal::dec!(100),
                unrealized_pl: rust_decimal::dec!(100.25),
                roi_percent: rust_decimal::dec!(11.14),
            }],
        }
    }

    async fn insert_summary(pool: &SqlitePool, summary: &PortfolioSummary) -> i64 {
        let mut tx = pool.begin().await.expect("begin transaction");

        let id = PortfolioSummaryRepository::insert(&mut tx, summary)
            .await
            .expect("insert portfolio summary");

        tx.commit().await.expect("commit transaction");

        id
    }

    #[tokio::test]
    async fn insert_then_find_by_id_returns_portfolio_summary() {
        let pool = setup_test_db().await;

        let summary = sample_portfolio_summary();

        let id = insert_summary(&pool, &summary).await;

        let loaded_summary = PortfolioSummaryRepository::find_by_id(&pool, id)
            .await
            .expect("find porfolio summary")
            .expect("porfolio summary exists");

        assert_eq!(loaded_summary.positions.len(), summary.positions.len());
        assert_eq!(loaded_summary.account_info.name, summary.account_info.name);
        assert_eq!(
            loaded_summary.totals.market_value,
            summary.totals.market_value
        );
    }

    #[tokio::test]
    async fn find_by_id_returns_none_for_missing_id() {
        let pool = setup_test_db().await;

        let result = PortfolioSummaryRepository::find_by_id(&pool, 123)
            .await
            .expect("database query should succeed");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn find_latest_returns_inserted_portfolio_summary() {
        let pool = setup_test_db().await;
        let summary = sample_portfolio_summary();

        insert_summary(&pool, &summary).await;

        let loaded = PortfolioSummaryRepository::find_latest(&pool)
            .await
            .expect("query latest portfolio summary")
            .expect("portfolio summary exists");

        assert_eq!(loaded.id, 1);
        assert_eq!(
            loaded.portfolio.account_info.name,
            summary.account_info.name
        );
        assert_eq!(
            loaded.portfolio.totals.market_value,
            summary.totals.market_value
        );
        assert_eq!(loaded.portfolio.positions.len(), summary.positions.len());
        assert_eq!(
            loaded.portfolio.positions[0].symbol,
            summary.positions[0].symbol
        );
    }

    #[tokio::test]
    async fn find_latest_returns_most_recently_inserted_summary() {
        let pool = setup_test_db().await;

        let first = sample_portfolio_summary();
        insert_summary(&pool, &first).await;

        let mut second = sample_portfolio_summary();
        second.account_info.name = Some("Latest Account".to_string());
        second.totals.market_value = rust_decimal::dec!(2000.75);
        second.positions[0].market_value = rust_decimal::dec!(2000.75);

        insert_summary(&pool, &second).await;

        let loaded = PortfolioSummaryRepository::find_latest(&pool)
            .await
            .expect("query latest portfolio summary")
            .expect("portfolio summary exists");

        assert_eq!(loaded.id, 2);
        assert_eq!(
            loaded.portfolio.account_info.name.as_deref(),
            Some("Latest Account")
        );
        assert_eq!(
            loaded.portfolio.totals.market_value,
            rust_decimal::dec!(2000.75)
        );
        assert_eq!(
            loaded.portfolio.positions[0].market_value,
            rust_decimal::dec!(2000.75)
        );
    }

    #[tokio::test]
    async fn find_latest_returns_none_in_empty_portfolio_summaries() {
        let pool = setup_test_db().await;

        let result = PortfolioSummaryRepository::find_latest(&pool)
            .await
            .expect("database query should succeed");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_returns_empty_for_empty_database() {
        let pool = setup_test_db().await;

        let reports = PortfolioSummaryRepository::list(&pool)
            .await
            .expect("list reports");

        assert!(reports.is_empty());
    }

    #[tokio::test]
    async fn list_returns_metadata_in_reverse_import_order() {
        let pool = setup_test_db().await;
        let first = sample_portfolio_summary();
        let first_id = insert_summary(&pool, &first).await;

        let mut second = sample_portfolio_summary();
        second.account_info.name = Some("Second Account".to_string());
        second.statement.period = Some("2026-05-01".to_string());
        let mut additional_position = second.positions[0].clone();
        additional_position.symbol = "VTI".to_string();
        second.positions.push(additional_position);
        second.totals.total_positions = second.positions.len();
        let second_id = insert_summary(&pool, &second).await;

        let first_created_at = "2026-06-03T10:00:00Z";
        let second_created_at = "2026-06-02T10:00:00Z";
        for (id, created_at) in [(first_id, first_created_at), (second_id, second_created_at)] {
            sqlx::query("UPDATE portfolio_summaries SET created_at = ? WHERE id = ?")
                .bind(created_at)
                .bind(id)
                .execute(&pool)
                .await
                .expect("set deterministic import timestamp");
        }

        let reports = PortfolioSummaryRepository::list(&pool)
            .await
            .expect("list reports");
        let expected = [
            (second_id, &second, second_created_at),
            (first_id, &first, first_created_at),
        ];

        assert_eq!(reports.len(), expected.len());
        for (report, (id, summary, created_at)) in reports.iter().zip(expected) {
            assert_eq!(report.id, id);
            assert_eq!(report.account_name, summary.account_info.name);
            assert_eq!(report.statement_period, summary.statement.period);
            assert_eq!(report.created_at, created_at);
            assert_eq!(report.total_positions, summary.totals.total_positions);
        }
    }

    #[tokio::test]
    async fn list_preserves_missing_account_name_and_statement_period() {
        let pool = setup_test_db().await;
        let mut summary = sample_portfolio_summary();
        summary.account_info.name = None;
        summary.statement.period = None;
        let id = insert_summary(&pool, &summary).await;

        let reports = PortfolioSummaryRepository::list(&pool)
            .await
            .expect("list reports with missing metadata");

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].id, id);
        assert!(reports[0].account_name.is_none());
        assert!(reports[0].statement_period.is_none());
    }

    #[tokio::test]
    async fn list_accepts_zero_total_positions() {
        let pool = setup_test_db().await;
        let mut summary = sample_portfolio_summary();
        summary.positions.clear();
        summary.totals.total_positions = 0;
        let id = insert_summary(&pool, &summary).await;

        let reports = PortfolioSummaryRepository::list(&pool)
            .await
            .expect("list reports without positions");

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].id, id);
        assert_eq!(reports[0].total_positions, 0);
    }

    #[tokio::test]
    async fn list_rejects_negative_total_positions() {
        let pool = setup_test_db().await;
        let summary = sample_portfolio_summary();
        let id = insert_summary(&pool, &summary).await;
        sqlx::query("UPDATE portfolio_summaries SET total_positions = -1 WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .expect("store invalid position count");

        let error = PortfolioSummaryRepository::list(&pool)
            .await
            .expect_err("negative position counts must be rejected");

        assert!(matches!(error, DbError::InvalidData(_)));
    }

    #[tokio::test]
    async fn delete_by_id_removes_report_and_positions_but_preserves_other_reports() {
        let pool = setup_test_db().await;
        let summary = sample_portfolio_summary();
        let remaining_id = insert_summary(&pool, &summary).await;
        let deleted_id = insert_summary(&pool, &summary).await;

        let is_deleted = PortfolioSummaryRepository::delete_by_id(&pool, deleted_id)
            .await
            .expect("delete report");
        assert!(is_deleted);

        let deleted = PortfolioSummaryRepository::find_by_id(&pool, deleted_id)
            .await
            .expect("look up deleted report");
        assert!(deleted.is_none());

        let deleted_position_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM portfolio_positions WHERE summary_id = ?",
        )
        .bind(deleted_id)
        .fetch_one(&pool)
        .await
        .expect("count positions of deleted report");
        assert_eq!(deleted_position_count, 0);

        let remaining = PortfolioSummaryRepository::find_by_id(&pool, remaining_id)
            .await
            .expect("look up remaining report")
            .expect("other report must remain");
        assert_eq!(remaining.account_info.name, summary.account_info.name);
        assert_eq!(remaining.positions.len(), summary.positions.len());
        assert_eq!(remaining.positions[0].symbol, summary.positions[0].symbol);

        let latest = PortfolioSummaryRepository::find_latest(&pool)
            .await
            .expect("find latest after deleting newest report")
            .expect("remaining report must become latest");
        assert_eq!(latest.id, remaining_id);
    }

    #[tokio::test]
    async fn delete_by_id_returns_false_for_missing_report() {
        let pool = setup_test_db().await;
        let summary = sample_portfolio_summary();
        let id = insert_summary(&pool, &summary).await;

        let is_deleted = PortfolioSummaryRepository::delete_by_id(&pool, id + 1)
            .await
            .expect("delete missing report");

        assert!(!is_deleted);
        let remaining = PortfolioSummaryRepository::find_by_id(&pool, id)
            .await
            .expect("look up existing report")
            .expect("existing report must remain");
        assert_eq!(remaining.positions.len(), summary.positions.len());
    }

    #[tokio::test]
    async fn delete_by_id_returns_false_when_report_is_already_deleted() {
        let pool = setup_test_db().await;
        let summary = sample_portfolio_summary();
        let id = insert_summary(&pool, &summary).await;

        assert!(PortfolioSummaryRepository::delete_by_id(&pool, id)
            .await
            .expect("delete existing report"));
        assert!(!PortfolioSummaryRepository::delete_by_id(&pool, id)
            .await
            .expect("delete report again"));
    }

    #[tokio::test]
    async fn delete_all_removes_reports_and_positions_and_returns_report_count() {
        let pool = setup_test_db().await;
        let first = sample_portfolio_summary();
        insert_summary(&pool, &first).await;

        let mut second = sample_portfolio_summary();
        let mut additional_position = second.positions[0].clone();
        additional_position.symbol = "VTI".to_string();
        second.positions.push(additional_position);
        second.totals.total_positions = second.positions.len();
        insert_summary(&pool, &second).await;

        let deleted_count = PortfolioSummaryRepository::delete_all(&pool)
            .await
            .expect("delete all reports");

        assert_eq!(deleted_count, 2);
        let report_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM portfolio_summaries")
            .fetch_one(&pool)
            .await
            .expect("count remaining reports");
        let position_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM portfolio_positions")
                .fetch_one(&pool)
                .await
                .expect("count remaining positions");
        assert_eq!(report_count, 0);
        assert_eq!(position_count, 0);
        assert!(PortfolioSummaryRepository::find_latest(&pool)
            .await
            .expect("find latest after clearing reports")
            .is_none());
        assert_eq!(
            PortfolioSummaryRepository::delete_all(&pool)
                .await
                .expect("clear reports again"),
            0,
        );
    }

    #[tokio::test]
    async fn delete_all_returns_zero_for_empty_database() {
        let pool = setup_test_db().await;

        let deleted_count = PortfolioSummaryRepository::delete_all(&pool)
            .await
            .expect("clear empty database");

        assert_eq!(deleted_count, 0);
    }

    #[tokio::test]
    async fn insert_after_delete_all_preserves_schema_and_keeps_ids_increasing() {
        let pool = setup_test_db().await;
        let summary = sample_portfolio_summary();
        let original_id = insert_summary(&pool, &summary).await;
        PortfolioSummaryRepository::delete_all(&pool)
            .await
            .expect("clear reports before reimport");

        let new_id = insert_summary(&pool, &summary).await;
        let reimported = PortfolioSummaryRepository::find_by_id(&pool, new_id)
            .await
            .expect("load reimported report")
            .expect("reimported report must exist");

        assert!(new_id > original_id);
        assert_eq!(reimported.account_info.name, summary.account_info.name);
        assert_eq!(reimported.totals.market_value, summary.totals.market_value);
        assert_eq!(reimported.positions.len(), summary.positions.len());
        assert_eq!(reimported.positions[0].symbol, summary.positions[0].symbol);
        assert!(PortfolioSummaryRepository::find_by_id(&pool, original_id)
            .await
            .expect("look up report removed before reimport")
            .is_none());
    }
}
