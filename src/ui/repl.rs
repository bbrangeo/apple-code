//! REPL loop and interactive commands

use anyhow::Result;
use rustyline::DefaultEditor;

use crate::api;
use crate::config::Config;
use crate::conversation::{format_history, ConversationManager};
use crate::types::GenerateRequest;
use crate::ui::{print_code_preview, Theme};

const HELP_TEXT: &str = r#"
Commands:
  /help      Show this help
  /exit      Exit the application
  /clear     Clear the screen
  /history   Show conversation history (session)
  /sessions  List saved sessions
  /models    List available models
  /languages List supported languages
  /test      Test API connection

Enter a prompt to generate code (e.g. "create a hello world in Python").
"#;

pub fn run(theme: Theme, config: &Config) -> Result<()> {
    let client = api::default_client();
    let mut manager = ConversationManager::new();
    let mut rl = DefaultEditor::new().map_err(|e| anyhow::anyhow!("readline: {}", e))?;
    let _ = rl.load_history(".apple-code-history");

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line.starts_with('/') {
                    if matches!(run_command(line, &mut manager)?, CommandResult::Exit) {
                        break;
                    }
                    continue;
                }
                if manager.current_session().is_none() {
                    manager.create_session();
                }
                let _ = manager.add_user_message(line);

                // Build conversational context from current session history (excluding the
                // just-entered line, which is already in `prompt`).
                let context = format_history(manager.history());

                let request = GenerateRequest {
                    prompt: line.to_string(),
                    language: config.default_language.clone(),
                    temperature: config.temperature.unwrap_or(0.7),
                    max_tokens: config.max_tokens.unwrap_or(4000),
                    model: config.model.clone(),
                    context: if context.is_empty() {
                        None
                    } else {
                        Some(context)
                    },
                    tool_mode: false,
                };
                match client.generate(&request) {
                    Ok(response) => {
                        let _ = manager.add_assistant_message(&response.code);
                        let _ = rl.add_history_entry(line);
                        print_code_preview(&response.code, response.language.as_deref(), theme);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(rustyline::error::ReadlineError::Eof)
            | Err(rustyline::error::ReadlineError::Interrupted) => break,
            Err(e) => return Err(e.into()),
        }
    }

    let _ = rl.save_history(".apple-code-history");
    Ok(())
}

enum CommandResult {
    Continue,
    Exit,
}

fn run_command(input: &str, manager: &mut ConversationManager) -> Result<CommandResult> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let cmd = parts.first().unwrap_or(&"").to_lowercase();
    match cmd.as_str() {
        "/help" => println!("{}", HELP_TEXT.trim()),
        "/exit" => return Ok(CommandResult::Exit),
        "/clear" => {
            print!("\x1b[2J\x1b[1;1H");
        }
        "/history" => {
            for msg in manager.history() {
                println!(
                    "[{}] {}",
                    msg.role,
                    msg.content.lines().next().unwrap_or("")
                );
            }
            if manager.history().is_empty() {
                println!("(No messages in current session)");
            }
        }
        "/sessions" => {
            if let Some(id) = parts.get(1) {
                match manager.load_session(id) {
                    Ok(Some(_)) => println!("Loaded session {}", id),
                    Ok(None) => println!("Session not found: {}", id),
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else {
                match manager.list_sessions() {
                    Ok(ids) => {
                        for id in &ids {
                            println!("  {}", id);
                        }
                        if ids.is_empty() {
                            println!("(No saved sessions)");
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        "/models" => println!("Available models: apple-foundation-model (mock)"),
        "/languages" => println!(
            "Supported: typescript, python, rust, go, java, ... (see 'apple-code languages')"
        ),
        "/test" => println!("API test: OK (mock client)"),
        _ => println!("Unknown command. Type /help for commands."),
    }
    Ok(CommandResult::Continue)
}
