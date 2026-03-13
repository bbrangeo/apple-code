//! CLI argument parser (clap)

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "apple-code")]
#[command(version, about = "Apple Code Assistant - Generate code using Apple Foundation Models")]
pub struct Args {
    /// Code generation prompt
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Programming language (typescript, python, etc.)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Interactive mode (default when no prompt provided)
    #[arg(short, long)]
    pub interactive: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Apple Foundation Model to use
    #[arg(short, long)]
    pub model: Option<String>,

    /// Temperature for code generation (0-2)
    #[arg(short, long, default_value = "0.7")]
    pub temperature: f32,

    /// Maximum tokens to generate
    #[arg(long, default_value = "4000")]
    pub max_tokens: u32,

    /// Save generated code to file
    #[arg(short, long)]
    pub save: bool,

    /// Copy generated code to clipboard
    #[arg(long)]
    pub copy: bool,

    /// Preview generated code in terminal
    #[arg(long)]
    pub preview: bool,

    /// Edit existing file with generated code
    #[arg(short, long)]
    pub edit: Option<String>,

    /// Additional context for code generation
    #[arg(long)]
    pub context: Option<String>,

    /// Terminal theme (light/dark)
    #[arg(long, default_value = "dark")]
    pub theme: String,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Verbose output
    #[arg(long)]
    pub verbose: bool,

    /// Debug mode
    #[arg(long)]
    pub debug: bool,

    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    /// Manage configuration
    Config {
        /// Set configuration value (key=value)
        #[arg(long)]
        set: Option<String>,
        /// Get configuration value
        #[arg(long)]
        get: Option<String>,
        /// List all configuration values
        #[arg(long)]
        list: bool,
        /// Reset configuration to defaults
        #[arg(long)]
        reset: bool,
    },
    /// List available Apple Foundation Models
    Models,
    /// List supported programming languages
    Languages,
    /// Test API connection
    Test,
}
