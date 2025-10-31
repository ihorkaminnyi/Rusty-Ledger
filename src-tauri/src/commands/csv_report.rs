use crate::{
    error::{BackendError, CommandError},
    infra::csv::ib_report_parser::IBCSVParser,
    portfolio::summary::PortfolioSummary,
};
use std::fs;
use std::path::{Path, PathBuf};
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
pub async fn process_csv_report(file_path: PathBuf) -> Result<PortfolioSummary, CommandError> {
    parse_portfolio_summary(&file_path).map_err(CommandError::from)
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
