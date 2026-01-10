//! Error types for Optio backend operations

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for Optio operations
#[derive(Error, Debug)]
pub enum OptioError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Script generation failed: {0}")]
    ScriptGeneration(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Client not found: {0}")]
    ClientNotFound(String),

    #[error("Encryption error: {0}")]
    Encryption(String),
}

/// Serializable error response for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl From<OptioError> for ErrorResponse {
    fn from(err: OptioError) -> Self {
        let code = match &err {
            OptioError::TemplateNotFound(_) => "TEMPLATE_NOT_FOUND",
            OptioError::InvalidConfig(_) => "INVALID_CONFIG",
            OptioError::ScriptGeneration(_) => "SCRIPT_GENERATION_FAILED",
            OptioError::Database(_) => "DATABASE_ERROR",
            OptioError::Io(_) => "IO_ERROR",
            OptioError::Serialization(_) => "SERIALIZATION_ERROR",
            OptioError::ClientNotFound(_) => "CLIENT_NOT_FOUND",
            OptioError::Encryption(_) => "ENCRYPTION_ERROR",
        };

        ErrorResponse {
            code: code.to_string(),
            message: err.to_string(),
            details: None,
        }
    }
}

// Implement Serialize for OptioError to work with Tauri commands
impl Serialize for OptioError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ErrorResponse::from(self.to_string()).serialize(serializer)
    }
}

impl From<String> for ErrorResponse {
    fn from(message: String) -> Self {
        ErrorResponse {
            code: "UNKNOWN_ERROR".to_string(),
            message,
            details: None,
        }
    }
}

impl From<rusqlite::Error> for OptioError {
    fn from(err: rusqlite::Error) -> Self {
        OptioError::Database(err.to_string())
    }
}

/// Result type alias for Optio operations
pub type OptioResult<T> = Result<T, OptioError>;
