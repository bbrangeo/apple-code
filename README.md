# Apple Code Assistant

Professional CLI tool powered by Apple Intelligence for on-device code generation. Features modern terminal UI, real-time streaming, multi-language support, and conversation management.

Inspired by [Apple-AI-CLI](https://github.com/0xatrilla/Apple-AI-CLI).

## Prerequisites

- **Rust** 1.70+ (edition 2021)
- **macOS 15+** with Apple Intelligence support (for future integration)
- **Apple Silicon Mac** recommended

## Build and run

```bash
cargo build
cargo run -- --help
```

### Examples

```bash
# Interactive mode (REPL with logo, /help, /exit, /history, /sessions)
cargo run -- -i

# Direct code generation with preview
cargo run -- -p "hello world in Python" -l python --preview

# Save to file
cargo run -- -p "quicksort" -l rust --save -o sort.rs

# Copy to clipboard
cargo run -- -p "utility function" --copy

# Edit existing file (send file content + prompt to model, write result back)
cargo run -- -p "add error handling" -e src/main.rs

# Configuration
cargo run -- config --list
cargo run -- config --get theme
cargo run -- config --set default_language=python
cargo run -- config --reset

# List models and languages, test API
cargo run -- models
cargo run -- languages
cargo run -- test
```

## Features

- **Config** – Load from `.env` and `~/.config/apple-code-assistant/config.toml`; subcommand `config --list/get/set/reset`
- **Terminal UI** – ASCII logo, theme (light/dark), bordered code preview
- **Code generation** – Trait-based client (mock included); direct mode with `-p` and options
- **REPL** – Interactive mode with `/help`, `/exit`, `/clear`, `/history`, `/sessions`, `/models`, `/languages`, `/test`
- **Conversations** – Sessions persisted under config dir; `/history` and `/sessions`
- **File operations** – `--save -o <file>`, `--edit <file>`
- **Clipboard** – `--copy` (arboard)
- **Syntax highlighting** – syntect in preview and REPL when language is set
- **Multi-language** – 20+ languages and aliases; `languages` subcommand; validation of `--language`
- **Signals** – Ctrl+C handled with “Operation cancelled”

## Development plan

A step-by-step development plan (phases 1–8) is in [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md).

## License

MIT
# apple-code
