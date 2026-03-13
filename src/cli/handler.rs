//! CLI execution handler

use std::io::{IsTerminal, Read};
use std::path::Path;

use anyhow::Result;

use crate::api::{self, CodeGenClient};
use crate::config::Config;
use crate::conversation::ConversationManager;
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
                        tool_mode: false,
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

        // Conversation manager for non-interactive CLI when extending conversations.
        let mut conversation_manager = if args.extend_conversation {
            let mut m = ConversationManager::new();
            // Try to load the most recent session if any.
            if let Ok(ids) = m.list_sessions() {
                if let Some(last) = ids.last() {
                    let _ = m.load_session(last);
                }
            }
            if m.current_session().is_none() {
                m.create_session();
            }
            Some(m)
        } else {
            None
        };

        // Apply prompt template from config if requested or if a default_prompt is configured.
        let prompt_template = config.resolve_prompt(args.template.as_deref());
        let effective_model = args
            .model
            .clone()
            .or_else(|| prompt_template.and_then(|(_, p)| p.model.clone()))
            .or_else(|| config.model.clone());
        let effective_temperature = prompt_template
            .and_then(|(_, p)| p.temperature)
            .unwrap_or(args.temperature);

        let client = api::default_client();
        let (request, edit_path) = if let Some(ref edit_file) = args.edit {
            let path = Path::new(edit_file);
            let content = utils::read_file(path).map_err(|e| anyhow::anyhow!("{}", e))?;

            // Build context: main edited file content + optional explicit context + globbed files.
            let mut context_parts: Vec<String> = Vec::new();
            context_parts.push(format!("File content ({}):\n{}", path.display(), content));
            if let Some(c) = args.context.as_deref() {
                context_parts.push(c.to_string());
            }
            if !args.context_glob.is_empty() {
                if let Some(glob_ctx) = read_context_globs(&args.context_glob) {
                    context_parts.push(glob_ctx);
                }
            }
            let mut context = context_parts.join("\n\n");
            if let Some(limit) = args.char_limit {
                if context.len() > limit as usize {
                    context.truncate(limit as usize);
                }
            }
            let request = GenerateRequest {
                prompt: args.prompt.as_ref().unwrap().clone(),
                language: language.clone(),
                temperature: effective_temperature,
                max_tokens: args.max_tokens,
                model: effective_model.clone(),
                context: Some(context),
                tool_mode: args.tool_mode,
            };
            (request, Some(path.to_path_buf()))
        } else {
            // When stdin is piped (not a TTY), read it and append it directly
            // to the prompt so the model clearly sees the input (smartcat-style).
            let mut piped_input = String::new();
            let has_piped_input = if !std::io::stdin().is_terminal() {
                std::io::stdin()
                    .read_to_string(&mut piped_input)
                    .is_ok()
                    && !piped_input.is_empty()
            } else {
                false
            };

            let mut prompt = args.prompt.as_ref().unwrap().clone();
            if has_piped_input {
                if let Some(limit) = args.char_limit {
                    if piped_input.len() > limit as usize {
                        piped_input.truncate(limit as usize);
                    }
                }
                prompt.push_str("\n\n");
                prompt.push_str(piped_input.trim_end());
            }

            // Start with any explicit --context value.
            let mut context_parts: Vec<String> = Vec::new();
            if let Some(ctx) = args.context.clone() {
                context_parts.push(ctx);
            }

            // Add glob-based context files, if any.
            if !args.context_glob.is_empty() {
                if let Some(glob_ctx) = read_context_globs(&args.context_glob) {
                    context_parts.push(glob_ctx);
                }
            }

            // If we are extending a conversation, add its history as context and
            // persist the new turn.
            if let Some(manager) = conversation_manager.as_mut() {
                let _ = manager.add_user_message(&prompt);
                let history = manager.history();
                if !history.is_empty() {
                    let mut history_ctx = String::new();
                    for msg in history {
                        let role = match msg.role {
                            crate::conversation::Role::User => "user",
                            crate::conversation::Role::Assistant => "assistant",
                        };
                        history_ctx.push_str("[");
                        history_ctx.push_str(role);
                        history_ctx.push_str("] ");
                        history_ctx.push_str(&msg.content);
                        history_ctx.push('\n');
                    }
                    context_parts.push(history_ctx);
                }
            }

            let context = if context_parts.is_empty() {
                None
            } else {
                let mut ctx = context_parts.join("\n\n");
                if let Some(limit) = args.char_limit {
                    if ctx.len() > limit as usize {
                        ctx.truncate(limit as usize);
                    }
                }
                Some(ctx)
            };

            let request = GenerateRequest {
                prompt,
                language: language.clone(),
                temperature: effective_temperature,
                max_tokens: args.max_tokens,
                model: effective_model,
                context,
                tool_mode: args.tool_mode,
            };
            (request, None)
        };
        let response = client.generate(&request).map_err(|e| anyhow::anyhow!("{}", e))?;
        let code = if request.tool_mode {
            strip_markdown_code_block(&response.code)
        } else {
            response.code.clone()
        };
        let theme = ui::Theme::from_str(&args.theme);
        let did_edit = edit_path.is_some();
        if let Some(ref path) = edit_path {
            utils::write_file(path, &code).map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("Wrote {}", path.display());
        }
        if let Some(ref out_path) = args.output {
            utils::write_file(Path::new(out_path), &code).map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("Wrote {}", out_path);
        }
        if args.copy {
            utils::copy_to_clipboard(&code).map_err(|e| anyhow::anyhow!("Clipboard: {}", e))?;
            println!("Copied to clipboard.");
        }
        if let Some(manager) = conversation_manager.as_mut() {
            let _ = manager.add_assistant_message(&code);
        }
        if args.preview || (args.output.is_none() && !did_edit) {
            if args.tool_mode {
                // In tool mode, emit plain output suitable for piping/editor integration.
                print!("{}", code);
            } else {
                if args.repeat_input && !std::io::stdin().is_terminal() {
                    // Repeat piped input followed by the model output, smartcat-style.
                    // We don't have the raw stdin anymore, but the effective prompt already
                    // contains it appended; repeating the prompt before the output keeps
                    // the behavior close enough for editor workflows.
                    println!("{}", args.prompt.as_deref().unwrap_or_default());
                    println!();
                }
                ui::print_code_preview(
                    &code,
                    response.language.as_deref(),
                    theme,
                );
            }
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

fn read_context_globs(patterns: &[String]) -> Option<String> {
    let mut chunks: Vec<String> = Vec::new();
    for pattern in patterns {
        if let Ok(paths) = glob::glob(pattern) {
            for entry in paths.flatten() {
                if entry.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&entry) {
                        chunks.push(format!("=== {} ===\n{}", entry.display(), content));
                    }
                }
            }
        }
    }
    if chunks.is_empty() {
        None
    } else {
        Some(chunks.join("\n\n"))
    }
}

fn strip_markdown_code_block(s: &str) -> String {
    // Find the first occurrence of triple backticks.
    let start_idx = match s.find("```") {
        Some(idx) => idx,
        None => return s.to_string(),
    };

    // Find the end of the opening fence line (to skip optional language tag).
    let after_fence = start_idx + 3;
    let rest = &s[after_fence..];
    let line_end_rel = match rest.find('\n') {
        Some(rel) => rel,
        None => return s.to_string(),
    };
    let content_start = after_fence + line_end_rel + 1;

    // Search for the closing fence from content_start.
    let rest_after = &s[content_start..];
    let end_rel = match rest_after.find("```") {
        Some(rel) => rel,
        None => return s.to_string(),
    };
    let content_end = content_start + end_rel;

    let mut block = s[content_start..content_end].to_string();
    // Trim leading and trailing whitespace/newlines.
    if block.starts_with('\n') {
        block = block[1..].to_string();
    }
    block.trim_end().to_string()
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}
