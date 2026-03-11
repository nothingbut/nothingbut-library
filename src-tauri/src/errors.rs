use serde::Serialize;
use std::io;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Module error: {0}")]
    Module(String),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Workspace("test workspace error".to_string());
        assert_eq!(err.to_string(), "Workspace error: test workspace error");
    }

    #[test]
    fn test_error_serialization() {
        let err = AppError::NotFound("item not found".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NotFound"));
        assert!(json.contains("item not found"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        match app_err {
            AppError::Io(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_result: Result<i32, _> = serde_json::from_str("invalid json");
        let json_err = json_result.unwrap_err();
        let app_err: AppError = json_err.into();
        match app_err {
            AppError::Json(_) => (),
            _ => panic!("Expected Json error"),
        }
    }
}
