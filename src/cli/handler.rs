//! CLI execution handler

use std::io::{IsTerminal, Read};
use std::path::Path;

use anyhow::Result;

use crate::api::{self, CodeGenClient};
use crate::config::Config;
use crate::error::ConfigError;
use crate::types::GenerateRequest;
use crate::ui;
use crate::utils::{self, is_supported, normalize, SUPPORTED_LANGUAGES};

use super::parser::{Args, Subcommand};

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, args: &Args, config: &Config) -> Result<()> {
        if let Some(ref sub) = args.subcommand {
            match sub {
                Subcommand::Config {
                    set,
                    get,
                    list,
                    reset,
                } => {
                    return self.run_config(config, args.config.as_deref(), *list, get.as_deref(), set.as_deref(), *reset);
                }
                Subcommand::Models => {
                    println!("Available models:");
                    println!("  apple-foundation-model (on-device, when available)");
                    println!("  (mock client used when API not configured)");
                }
                Subcommand::Languages => {
                    println!("Supported languages:");
                    for (name, aliases) in SUPPORTED_LANGUAGES {
                        if aliases.is_empty() {
                            println!("  {}", name);
                        } else {
                            println!("  {} (aliases: {})", name, aliases.join(", "));
                        }
                    }
                }
                Subcommand::Test => {
                    let client = api::default_client();
                    let req = GenerateRequest {
                        prompt: "test".to_string(),
                        language: Some("rust".to_string()),
                        temperature: 0.7,
                        max_tokens: 10,
                        model: None,
                        context: None,
                    };
                    match client.generate(&req) {
                        Ok(_) => println!("API test: OK"),
                        Err(e) => eprintln!("API test failed: {}", e),
                    }
                }
            }
            return Ok(());
        }

        if args.interactive || args.prompt.is_none() {
            let theme = ui::Theme::from_str(&args.theme);
            ui::run_interactive(theme, config)?;
            return Ok(());
        }

        if let Some(ref lang) = args.language {
            if !is_supported(lang) {
                return Err(anyhow::anyhow!("Unsupported language: '{}'. Use 'apple-code languages' to list supported languages.", lang));
            }
        }
        let language = args
            .language
            .as_ref()
            .and_then(|l| normalize(l).or(Some(l.clone())))
            .or_else(|| config.default_language.clone());
        let client = api::default_client();
        let (request, edit_path) = if let Some(ref edit_file) = args.edit {
            let path = Path::new(edit_file);
            let content = utils::read_file(path).map_err(|e| anyhow::anyhow!("{}", e))?;
            let context = args.context.as_deref().map_or(content.clone(), |c| format!("{}\n\nFile content:\n{}", c, content));
            let request = GenerateRequest {
                prompt: args.prompt.as_ref().unwrap().clone(),
                language: language.clone(),
                temperature: args.temperature,
                max_tokens: args.max_tokens,
                model: args.model.clone().or_else(|| config.model.clone()),
                context: Some(context),
            };
            (request, Some(path.to_path_buf()))
        } else {
            let context = if !std::io::stdin().is_terminal() {
                let mut stdin_content = String::new();
                if std::io::stdin().read_to_string(&mut stdin_content).is_ok() && !stdin_content.is_empty() {
                    Some(match &args.context {
                        Some(c) => format!("{}\n\n{}", stdin_content.trim_end(), c),
                        None => stdin_content,
                    })
                } else {
                    args.context.clone()
                }
            } else {
                args.context.clone()
            };
            let request = GenerateRequest {
                prompt: args.prompt.as_ref().unwrap().clone(),
                language: language.clone(),
                temperature: args.temperature,
                max_tokens: args.max_tokens,
                model: args.model.clone().or_else(|| config.model.clone()),
                context,
            };
            (request, None)
        };
        let response = client.generate(&request).map_err(|e| anyhow::anyhow!("{}", e))?;
        let theme = ui::Theme::from_str(&args.theme);
        let did_edit = edit_path.is_some();
        if let Some(ref path) = edit_path {
            utils::write_file(path, &response.code).map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("Wrote {}", path.display());
        }
        if let Some(ref out_path) = args.output {
            utils::write_file(Path::new(out_path), &response.code).map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("Wrote {}", out_path);
        }
        if args.copy {
            utils::copy_to_clipboard(&response.code).map_err(|e| anyhow::anyhow!("Clipboard: {}", e))?;
            println!("Copied to clipboard.");
        }
        if args.preview || (args.output.is_none() && !did_edit) {
            ui::print_code_preview(
                &response.code,
                response.language.as_deref(),
                theme,
            );
        }
        Ok(())
    }

    fn run_config(
        &self,
        config: &Config,
        config_file_override: Option<&str>,
        list: bool,
        get: Option<&str>,
        set: Option<&str>,
        reset: bool,
    ) -> Result<()> {
        if reset {
            let default = Config::default();
            default.save(config_file_override.map(Path::new))?;
            println!("Configuration reset to defaults.");
            return Ok(());
        }
        if let Some(s) = set {
            let mut c = config.clone();
            let (key, value) = s
                .split_once('=')
                .ok_or_else(|| ConfigError::Invalid("expected key=value".to_string()))?;
            c.set(key.trim(), value.trim())?;
            c.save(config_file_override.map(Path::new))?;
            println!("Set {} = {}", key.trim(), value.trim());
            return Ok(());
        }
        if let Some(key) = get {
            match config.get(key) {
                Some(v) => println!("{}", v),
                None => return Err(ConfigError::Invalid(format!("unknown key: {}", key)).into()),
            }
            return Ok(());
        }
        if list {
            for key in Config::keys() {
                let val = config.get(key).unwrap_or_else(|| "<unset>".to_string());
                println!("{} = {}", key, val);
            }
            if let Some(ref path) = config.config_file {
                println!("(config file: {})", path.display());
            }
            return Ok(());
        }
        println!("Use --list, --get <key>, --set key=value, or --reset.");
        Ok(())
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}
