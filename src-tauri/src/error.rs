use rust_decimal::Error as DecimalError;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("CSV reading error")]
    Csv(#[from] csv::Error),
    #[error("Invalid number format: {0}")]
    InvalidNumber(#[from] DecimalError),
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("File not found")]
    FileNotFound { path: String },

    #[error("Failed to read file")]
    FileRead {
        #[source]
        source: std::io::Error,
    },

    #[error("Report parsing failed")]
    ParseFailed {
        #[from]
        source: ParseError,
    },

    #[error("Validation failed")]
    Validation { reason: String },
}

impl From<BackendError> for CommandError {
    fn from(value: BackendError) -> Self {
        match value {
            BackendError::FileNotFound { path } => CommandError::new(
                "file_not_found",
                "File not found. Please check the path and try again.",
            )
            .with_details(path),
            BackendError::FileRead { source } => CommandError::new(
                "file_read_failed",
                "The selected file could not be opened. Please check file permissions.",
            )
            .with_details(source.to_string()),
            BackendError::ParseFailed { source } => {
                CommandError::new("report_parse_failed", "Could not parse the report.")
                    .with_details(source.to_string())
            }
            BackendError::Validation { reason } => CommandError::new("validation_failed", reason),
        }
    }
}
