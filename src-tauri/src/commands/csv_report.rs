use crate::{infra::csv::ib_report_parser::IBCSVParser, portfolio::summary::PortfolioSummary};
use std::fs;
use std::path::Path;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn open_file_dialog(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app_handle
        .dialog()
        .file()
        .add_filter("CSV Files", &["csv"])
        .set_title("Select Interactive Brokers CSV Report")
        .blocking_pick_file();

    match file_path {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn process_csv_report(file_path: String) -> Result<PortfolioSummary, String> {
    parse_portfolio_summary(&file_path)
}

pub fn parse_portfolio_summary(file_path: &str) -> Result<PortfolioSummary, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("File not found. Please check the file path and try again.".to_string());
    }

    let raw_content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            return Err("The selected file could not be opened. Please check the file permissions and try again.".to_string());
        }
    };
    IBCSVParser::parse_report_view(&raw_content)
        .map_err(|err| format!("Failed to parse report: {err}"))
        .map(PortfolioSummary::from)
}
