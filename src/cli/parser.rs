//! CLI argument parser (clap)

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "apple-code")]
#[command(
    version,
    about = "Apple Code Assistant - Generate code using Apple Foundation Models"
)]
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

    /// Prompt template name (from config)
    #[arg(short = 'T', long)]
    pub template: Option<String>,

    /// Temperature for code generation (0-2)
    #[arg(short, long, default_value = "0.7")]
    pub temperature: f32,

    /// Maximum tokens to generate
    #[arg(long, default_value = "4000")]
    pub max_tokens: u32,

    /// Maximum number of characters to use for piped stdin and context
    #[arg(long)]
    pub char_limit: Option<usize>,

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

    /// Glob patterns for additional context files
    #[arg(long = "context-glob")]
    pub context_glob: Vec<String>,

    /// Terminal theme (light/dark)
    #[arg(long, default_value = "dark")]
    pub theme: String,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Verbose output, including HTTP server request logs (alias: --logs)
    #[arg(long, alias = "logs", global = true)]
    pub verbose: bool,

    /// Debug mode
    #[arg(long)]
    pub debug: bool,

    /// Extend previous conversation instead of starting a new one
    #[arg(long)]
    pub extend_conversation: bool,

    /// Repeat input before output (useful for editor integration)
    #[arg(short = 'r', long)]
    pub repeat_input: bool,

    /// Raw/tool mode: print plain output without preview formatting
    #[arg(long)]
    pub tool_mode: bool,

    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
}

impl Args {
    /// Whether this invocation starts the HTTP server.
    pub fn is_serving(&self) -> bool {
        matches!(self.subcommand, Some(Subcommand::Serve { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::{CommandFactory, Parser};

    #[test]
    fn cli_options_are_valid() {
        Args::command().debug_assert();
    }

    #[test]
    fn server_logs_enable_verbose_output() {
        let args =
            Args::try_parse_from(["apple-code", "serve", "--logs"]).expect("server accepts --logs");
        assert!(args.verbose);
        assert!(args.is_serving());
        let args =
            Args::try_parse_from(["apple-code", "serve"]).expect("server accepts default options");
        assert!(!args.verbose);
    }
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
    /// Start an OpenAI-compatible HTTP server (requires the server feature)
    Serve {
        /// Bind address; non-loopback addresses expose the unauthenticated API
        #[arg(long, default_value = "127.0.0.1")]
        host: std::net::IpAddr,
        /// TCP port
        #[arg(long, default_value_t = 11434, value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,
    },
    /// List available Apple Foundation Models
    Models,
    /// List supported programming languages
    Languages,
    /// Test API connection
    Test,
}
