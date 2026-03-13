//! Apple Code Assistant - Main entry point
//! CLI for code generation using Apple Foundation Models

use anyhow::Result;
use clap::Parser;

use apple_code_assistant::config::Config;
use apple_code_assistant::utils;
use apple_code_assistant::{Args, Handler};

fn main() -> Result<()> {
    ctrlc::set_handler(|| {
        eprintln!("\nOperation cancelled by user");
        std::process::exit(130);
    })?;
    let args = Args::parse();
    utils::setup_logging(args.verbose, args.debug);
    let config = Config::load(args.config.as_deref()).map_err(|e| anyhow::anyhow!("{}", e))?;
    let handler = Handler::new();
    handler.run(&args, &config)
}
