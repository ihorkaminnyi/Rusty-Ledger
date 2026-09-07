use crate::{
    error::{BackendError, CommandError, DbError},
    infra::{
        csv::ib_report_parser::IBCSVParser,
        persistence::repositories::portfolio_summary_repository::PortfolioSummaryRepository,
    },
    portfolio::summary::PortfolioSummary,
    AppState,
};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn open_file_dialog(
    app_handle: tauri::AppHandle,
) -> Result<Option<PathBuf>, CommandError> {
    let file_path = app_handle
        .dialog()
        .file()
        .add_filter("CSV Files", &["csv"])
        .set_title("Select Interactive Brokers CSV Report")
        .blocking_pick_file();

    file_path
        .map(|path| {
            path.simplified().into_path().map_err(|error| {
                CommandError::new(
                    "invalid_file_path",
                    "The selected file path could not be resolved.",
                )
                .with_details(error.to_string())
            })
        })
        .transpose()
}

#[tauri::command]
pub async fn process_csv_report(
    state: State<'_, AppState>,
    file_path: PathBuf,
) -> Result<PortfolioSummary, CommandError> {
    let summary = parse_portfolio_summary(&file_path).map_err(CommandError::from)?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(DbError::Query)
        .map_err(BackendError::from)
        .map_err(CommandError::from)?;

    PortfolioSummaryRepository::insert(&mut tx, &summary)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)?;

    tx.commit()
        .await
        .map_err(DbError::Query)
        .map_err(BackendError::from)
        .map_err(CommandError::from)?;

    Ok(summary)
}

#[tauri::command]
pub async fn get_latest_portfolio_summary(
    state: State<'_, AppState>,
) -> Result<Option<PortfolioSummary>, CommandError> {
    let latest_summary = PortfolioSummaryRepository::find_latest(&state.db)
        .await
        .map_err(BackendError::from)
        .map_err(CommandError::from)?;

    Ok(latest_summary)
}

pub fn parse_portfolio_summary<P: AsRef<Path>>(
    file_path: P,
) -> Result<PortfolioSummary, BackendError> {
    let path = file_path.as_ref();
    let raw_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(BackendError::FileNotFound {
                path: path.to_string_lossy().into_owned(),
            })
        }
        Err(source) => return Err(BackendError::FileRead { source }),
    };

    let report_view = IBCSVParser::parse_report_view(&raw_content)?;
    PortfolioSummary::try_from_report(report_view).map_err(BackendError::from)
}
