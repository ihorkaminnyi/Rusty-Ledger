use tauri::State;

use crate::{
    error::{BackendError, CommandError},
    infra::persistence::repositories::portfolio_summary_repository::PortfolioSummaryRepository,
    portfolio::summary::{StoredPortfolioSummary, StoredReportListItem},
    AppState,
};

#[tauri::command]
pub async fn list_portfolio_summaries(
    state: State<'_, AppState>,
) -> Result<Vec<StoredReportListItem>, CommandError> {
    PortfolioSummaryRepository::list(&state.db)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_portfolio_summary(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<StoredPortfolioSummary>, CommandError> {
    let portfolio = PortfolioSummaryRepository::find_by_id(&state.db, id)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)?;

    Ok(portfolio.map(|portfolio| StoredPortfolioSummary { id, portfolio }))
}

#[tauri::command]
pub async fn delete_portfolio_summary(
    state: State<'_, AppState>,
    id: i64,
) -> Result<bool, CommandError> {
    PortfolioSummaryRepository::delete_by_id(&state.db, id)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn clear_portfolio_summaries(state: State<'_, AppState>) -> Result<u64, CommandError> {
    PortfolioSummaryRepository::delete_all(&state.db)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)
}
