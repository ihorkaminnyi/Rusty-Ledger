use rust_decimal::Decimal;
use sqlx::SqlitePool;
use tauri::State;

use crate::{
    error::{BackendError, CommandError},
    infra::persistence::repositories::portfolio_summary_repository::PortfolioSummaryRepository,
    portfolio::rebalance::{
        PortfolioRebalancer, RebalancePlan, RebalanceStrategy, TargetAllocation, ValidatedTargets,
    },
    AppState,
};

use super::csv_report::parse_portfolio_summary;

#[tauri::command]
pub async fn suggest_rebalance(
    file_path: String,
    targets: Vec<TargetAllocation>,
    deposit_amount: f64,
    rebalance_strategy: RebalanceStrategy,
) -> Result<RebalancePlan, CommandError> {
    let validated_targets = ValidatedTargets::new(targets).map_err(CommandError::from)?;
    let deposit = Decimal::from_f64_retain(deposit_amount).unwrap_or(Decimal::ZERO);

    let summary = parse_portfolio_summary(&file_path).map_err(CommandError::from)?;
    Ok(PortfolioRebalancer::calculate(
        &summary,
        &validated_targets,
        deposit,
        rebalance_strategy,
    ))
}

#[tauri::command]
pub async fn suggest_rebalance_for_summary(
    state: State<'_, AppState>,
    id: i64,
    targets: Vec<TargetAllocation>,
    deposit_amount: f64,
    rebalance_strategy: RebalanceStrategy,
) -> Result<RebalancePlan, CommandError> {
    rebalance_for_summary(&state.db, id, targets, deposit_amount, rebalance_strategy).await
}

async fn rebalance_for_summary(
    pool: &SqlitePool,
    id: i64,
    targets: Vec<TargetAllocation>,
    deposit_amount: f64,
    rebalance_strategy: RebalanceStrategy,
) -> Result<RebalancePlan, CommandError> {
    let validated_targets = ValidatedTargets::new(targets).map_err(CommandError::from)?;
    let summary = PortfolioSummaryRepository::find_by_id(pool, id)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)?
        .ok_or_else(|| {
            CommandError::new("report_not_found", "The saved report no longer exists.")
        })?;
    let deposit = Decimal::from_f64_retain(deposit_amount).unwrap_or(Decimal::ZERO);

    Ok(PortfolioRebalancer::calculate(
        &summary,
        &validated_targets,
        deposit,
        rebalance_strategy,
    ))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    use super::*;
    use crate::portfolio::summary::PortfolioSummary;

    async fn setup_saved_report() -> (SqlitePool, i64, PortfolioSummary) {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("valid database URL")
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("connect test database");
        sqlx::migrate!("./src/infra/persistence/migrations")
            .run(&pool)
            .await
            .expect("apply test migrations");

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/test_report.csv");
        let summary = parse_portfolio_summary(path).expect("parse sample CSV");
        let mut tx = pool.begin().await.expect("begin transaction");
        let id = PortfolioSummaryRepository::insert(&mut tx, &summary)
            .await
            .expect("save sample report");
        tx.commit().await.expect("commit sample report");
        (pool, id, summary)
    }

    fn targets(summary: &PortfolioSummary) -> Vec<TargetAllocation> {
        vec![TargetAllocation {
            symbol: summary.positions[0].symbol.clone(),
            target_percent: rust_decimal::dec!(100),
        }]
    }

    #[tokio::test]
    async fn saved_report_rebalancing_matches_calculation_from_imported_csv() {
        let (pool, id, summary) = setup_saved_report().await;

        for strategy in [RebalanceStrategy::BuyOnly, RebalanceStrategy::Full] {
            let validated_targets =
                ValidatedTargets::new(targets(&summary)).expect("valid targets");
            let mut expected = PortfolioRebalancer::calculate(
                &summary,
                &validated_targets,
                rust_decimal::dec!(500),
                strategy.clone(),
            );
            let mut actual = rebalance_for_summary(&pool, id, targets(&summary), 500.0, strategy)
                .await
                .expect("calculate from the database without a CSV path");
            expected.trades.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            actual.trades.sort_by(|a, b| a.symbol.cmp(&b.symbol));

            assert_eq!(
                serde_json::to_value(actual).expect("serialize actual plan"),
                serde_json::to_value(expected).expect("serialize expected plan"),
            );
        }
    }

    #[tokio::test]
    async fn deleted_or_missing_report_returns_report_not_found() {
        let (pool, id, summary) = setup_saved_report().await;
        PortfolioSummaryRepository::delete_by_id(&pool, id)
            .await
            .expect("delete saved report");

        for missing_id in [id, id + 1] {
            let error = rebalance_for_summary(
                &pool,
                missing_id,
                targets(&summary),
                0.0,
                RebalanceStrategy::BuyOnly,
            )
            .await
            .expect_err("unavailable reports must be rejected");

            assert_eq!(error.code, "report_not_found");
        }
    }

    #[tokio::test]
    async fn saved_report_rebalancing_rejects_invalid_targets() {
        let (pool, id, _) = setup_saved_report().await;

        let error = rebalance_for_summary(&pool, id, vec![], 0.0, RebalanceStrategy::BuyOnly)
            .await
            .expect_err("empty targets must be rejected");

        assert_eq!(error.code, "validation_failed");
    }
}
