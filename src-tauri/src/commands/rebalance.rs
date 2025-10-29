use crate::{
    error::CommandError,
    portfolio::rebalance::{
        PortfolioRebalancer, RebalancePlan, TargetAllocation, ValidatedTargets,
    },
};

use super::csv_report::parse_portfolio_summary;

#[tauri::command]
pub async fn suggest_rebalance(
    file_path: String,
    targets: Vec<TargetAllocation>,
) -> Result<RebalancePlan, CommandError> {
    let validated_targets = ValidatedTargets::new(targets).map_err(CommandError::from)?;

    let summary = parse_portfolio_summary(&file_path).map_err(CommandError::from)?;
    Ok(PortfolioRebalancer::calculate(&summary, &validated_targets))
}
