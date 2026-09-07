use rust_decimal::Decimal;

use crate::{
    error::CommandError,
    portfolio::rebalance::{
        PortfolioRebalancer, RebalancePlan, RebalanceStrategy, TargetAllocation, ValidatedTargets,
    },
};

use super::csv_report::parse_portfolio_summary;

// TODO: calcute rebalance from the latest portfolio summary
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
