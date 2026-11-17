//! Utilities: logging, clipboard, file operations, syntax highlighting

mod languages;
mod syntax_highlight;

use std::path::Path;

use crate::error::IoError;

pub use languages::{from_extension, from_prompt, is_supported, normalize, SUPPORTED_LANGUAGES};
pub use syntax_highlight::highlight_to_ansi;

/// Initialize logging from verbosity and debug flags.
/// Call once at startup after parsing args.
pub fn setup_logging(verbose: bool, debug: bool) {
    let level = if debug {
        log::LevelFilter::Debug
    } else if verbose {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Warn
    };
    let _ = env_logger::Builder::from_default_env()
        .filter_level(level)
        .try_init();
}

/// Write content to a file. Creates parent dirs if needed.
pub fn write_file(path: &Path, content: &str) -> Result<(), IoError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(IoError::Io)?;
    }
    std::fs::write(path, content).map_err(IoError::Io)?;
    Ok(())
}

/// Read entire file as string.
pub fn read_file(path: &Path) -> Result<String, IoError> {
    std::fs::read_to_string(path).map_err(IoError::Io)
}

/// Copy text to system clipboard (macOS/Windows/Linux via arboard).
pub fn copy_to_clipboard(text: &str) -> Result<(), IoError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| IoError::PermissionDenied(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| IoError::PermissionDenied(e.to_string()))?;
    Ok(())
}
