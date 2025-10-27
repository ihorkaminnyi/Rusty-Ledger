use crate::portfolio::rebalance::{PortfolioRebalancer, RebalancePlan, TargetAllocation};

use super::csv_report::parse_portfolio_summary;

#[tauri::command]
pub async fn suggest_rebalance(
    file_path: String,
    targets: Vec<TargetAllocation>,
) -> Result<RebalancePlan, String> {
    if targets.is_empty() {
        return Err("Target allocations are required to compute a rebalance plan.".to_string());
    }

    let summary = parse_portfolio_summary(&file_path)?;
    Ok(PortfolioRebalancer::calculate(&summary, &targets))
}
