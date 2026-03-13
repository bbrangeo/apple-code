//! Application error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("invalid config: {0}")]
    Invalid(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("API error: {0}")]
    Request(String),
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("timeout")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum IoError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
