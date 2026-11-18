//! Apple Code Assistant - Library root
//! CLI for code generation using Apple Foundation Models

mod api;
mod cli;
pub mod config;
mod conversation;
mod error;
#[cfg(feature = "server")]
mod server;
mod types;
mod ui;
pub mod utils;

pub use error::{ApiError, ConfigError, IoError};

pub use cli::{Args, Handler};
