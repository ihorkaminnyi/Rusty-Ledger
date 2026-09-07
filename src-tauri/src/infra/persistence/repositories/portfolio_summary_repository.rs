use std::str::FromStr;

use rust_decimal::Decimal;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::{
    error::DbError,
    infra::csv::ib_report_parser::view::{AccountInfo, StatementInfo},
    portfolio::summary::{PortfolioSummary, PortfolioTotals, PositionSummary},
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

    pub async fn find_latest(pool: &SqlitePool) -> Result<Option<PortfolioSummary>, DbError> {
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

        Self::find_by_id(pool, latest_id).await
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

        assert_eq!(loaded.account_info.name, summary.account_info.name);
        assert_eq!(loaded.totals.market_value, summary.totals.market_value);
        assert_eq!(loaded.positions.len(), summary.positions.len());
        assert_eq!(loaded.positions[0].symbol, summary.positions[0].symbol);
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

        assert_eq!(loaded.account_info.name.as_deref(), Some("Latest Account"),);
        assert_eq!(loaded.totals.market_value, rust_decimal::dec!(2000.75),);
    }

    #[tokio::test]
    async fn find_latest_returns_none_in_empty_portfolio_summaries() {
        let pool = setup_test_db().await;

        let result = PortfolioSummaryRepository::find_latest(&pool)
            .await
            .expect("database query should succeed");

        assert!(result.is_none());
    }
}
