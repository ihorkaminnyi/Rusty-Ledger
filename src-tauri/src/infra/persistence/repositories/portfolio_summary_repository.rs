use std::str::FromStr;

use rust_decimal::Decimal;
use sqlx::{Sqlite, Transaction};

use crate::{
    error::DbError,
    infra::csv::ib_report_parser::{
        parsers::statement,
        view::{AccountInfo, StatementInfo},
    },
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
        pool: &sqlx::SqlitePool,
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
}

fn parse_decimal(value: &str, field: &'static str) -> Result<Decimal, DbError> {
    Decimal::from_str(value).map_err(|source| DbError::InvalidDecimal { field, source })
}
