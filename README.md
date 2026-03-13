# Apple Code Assistant

Professional CLI tool powered by Apple Intelligence for on-device code generation. Features modern terminal UI, real-time streaming, multi-language support, and conversation management.

Inspired by [Apple-AI-CLI](https://github.com/0xatrilla/Apple-AI-CLI).

## Prerequisites

- **Rust** 1.70+ (edition 2021)
- **macOS 15+** with Apple Intelligence support (for future integration)
- **Apple Silicon Mac** recommended

## Installation

```bash
cargo install apple-code-assistant
```

Then use the `apple-code` binary:

```bash
apple-code --help
```

## Build and run

```bash
cargo build
cargo run -- --help
```

### Quick start

```bash
# Simple generation
apple-code -p "hello world in Python" -l python --tool-mode

# Summarize a diff (smartcat-style)
git diff | apple-code -p "summarize the changes" --tool-mode --char-limit 8000

# Use a template for tests
apple-code -p "write tests for this function" -T tests --tool-mode
```

### Getting plain code only (no markdown)

By default, some models may still wrap code in ``` fences and add explanations.
To force plain code output suitable for files and editors:

```bash
# Only code written to the file (no markdown, no explanation)
apple-code -p "implement binary search" -l python --save -o binary_search.py --tool-mode
```

You can further tighten behavior with a prompt template in `config.toml`, for example:

```toml
[prompts.strict_code]
system = "Never use markdown or explanations. Return only the final code."
```

Then call:

```bash
apple-code -p "implement binary search" -l python -T strict_code --save -o binary_search.py --tool-mode
```

### Examples (from source checkout)

```bash
# Interactive mode (REPL with logo, /help, /exit, /history, /sessions)
cargo run -- -i

# Direct code generation with preview
cargo run -- -p "hello world in Python" -l python --preview

# Save to file
cargo run -- -p "quicksort" -l rust --save -o sort.rs

# Copy to clipboard
cargo run -- -p "utility function" --copy

# Use stdin as context (pipe data in, smartcat-style)
git diff | apple-code -p "summarize the changes"
git diff | apple-code -p "summarize the changes" > summary.txt
cat src/main.rs | apple-code -p "add doc comments to this code"

# Use prompt templates from config.toml
apple-code -p "write tests for this function" -T tests
apple-code -p "refactor this module" -T refactor -l rust

# Extend previous conversation in non-interactive mode
apple-code -p "summarize the architecture of this project" --extend-conversation
apple-code -p "now propose improvements" --extend-conversation

# Repeat input before output (useful in editors)
git diff | apple-code -p "apply these changes but fix formatting" --repeat-input --tool-mode

# Limit size of piped/context data and add globbed context files
git diff | apple-code -p "summarize the changes" --char-limit 8000 --context-glob "src/**/*.rs"

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
- **Stdin piping** – When stdin is not a TTY (e.g. `git diff | apple-code -p "…"`) the piped content is appended to the prompt (smartcat-style)
- **Prompt templates** – Templates configurables dans `config.toml`, sélectionnés via `--template/-T`
- **CLI conversations** – `--extend-conversation` pour réutiliser la dernière session, `--repeat-input` pour rejouer l’entrée avant la sortie
- **Context control** – `--char-limit` pour borner la taille de stdin/contexte, `--context-glob` pour ajouter des fichiers de contexte par glob
- **Tool mode** – `--tool-mode` pour une sortie brute adaptée aux éditeurs (aucun cadre ni mise en forme)
- **Signals** – Ctrl+C handled with “Operation cancelled”

## Development plan

A step-by-step development plan (phases 1–8) is in [docs/DEVELOPMENT_PLAN.md](docs/DEVELOPMENT_PLAN.md).

## License

MIT

# apple-code
