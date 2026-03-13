//! Terminal UI: branding, theme, code preview, REPL

mod branding;
mod repl;

pub use branding::{print_code_preview, print_logo, Theme};

use anyhow::Result;

use crate::config::Config;

/// Run interactive mode (REPL). Shows logo then runs the command loop.
pub fn run_interactive(theme: Theme, config: &Config) -> Result<()> {
    print_logo(theme);
    repl::run(theme, config)
}
