# Apple Code Assistant

Professional CLI tool powered by Apple Intelligence for on-device code generation. Features modern terminal UI, real-time streaming, multi-language support, and conversation management.

Inspired by [Apple-AI-CLI](https://github.com/0xatrilla/Apple-AI-CLI).

## Prerequisites

- **Rust** 1.70+ (edition 2021)
- **macOS 26+** with Apple Intelligence enabled and its on-device model downloaded
- **Apple Silicon Mac**
- **Xcode 26+** or compatible Command Line Tools with `swiftc` available

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

To replace an installed `apple-code` with the version from this checkout:

```bash
cargo install --path . --force
apple-code --version
```

The checkout sets `MACOSX_DEPLOYMENT_TARGET=26.0` in `.cargo/config.toml`,
required by the Swift bridge. For a crates.io installation, set it explicitly:

```bash
MACOSX_DEPLOYMENT_TARGET=26.0 cargo install apple-code-assistant --force
```

### Quick start

```bash
# Simple generation
apple-code -p "hello world in Python" -l python --tool-mode

# Summarize a diff (smartcat-style)
git diff | apple-code -p "summarize the changes" --tool-mode --char-limit 2000 --max-tokens 512

# Use a template for tests
apple-code -p "write tests for this function" -T tests --tool-mode
```

### Piped input and context limits

In tool mode, omit `-l` for text tasks such as summaries; the configured default
language is only applied outside tool mode. Use `-l python` (or another language)
when generating code.

The on-device model has a limited context window, including instructions and
conversation history. If a large diff fails, reduce `--char-limit` and
`--max-tokens`. These are input/output caps, not a guarantee that a request fits.
`--char-limit` truncates the input: a summary then covers only that portion of
the diff. For a full review, process smaller diffs separately.

### Getting plain code only (no markdown)

By default, some models may still wrap code in ``` fences and add explanations.
When saving, editing, or redirecting source code, specify its language with `-l`
and use `--tool-mode` to request raw code and remove Markdown fences. This does
not validate the generated program: review and test it before use.

```bash
# Only code written to the file (no markdown, no explanation)
apple-code -p "implement binary search" -l python --save -o binary_search.py --tool-mode
```

You can further tighten behavior with a prompt template in `config.toml`, for example:

```toml
[prompts.strict_code]
system = "Never use markdown or explanations. Return only the final code."
```

Then call (use `--config config.toml` to load the file from this checkout;
otherwise configuration is read from the user config directory):

```bash
apple-code --config config.toml -p "implement binary search" -l python -T strict_code --save -o binary_search.py --tool-mode
```

### Examples (from source checkout)

```bash
# Interactive mode (REPL with logo, /help, /exit, /history, /sessions)
cargo run -- -i

# Interactive Rust session: -l overrides the configured default for this session
apple-code -l rust

# Direct code generation with preview
cargo run -- -p "hello world in Python" -l python --preview

# Save raw Rust code, without terminal formatting or Markdown fences
cargo run -- -p "Implement pub fn sort_numbers(values: &mut [i32]) by calling values.sort_unstable()." -l rust --save -o sort.rs --tool-mode
rustc --edition 2021 --crate-type lib --emit metadata sort.rs

# Copy to clipboard
cargo run -- -p "utility function" --copy

# Use stdin as context (pipe data in, smartcat-style)
git diff | apple-code -p "summarize the changes"
git diff | apple-code -p "summarize the changes" > summary.txt
cat src/main.rs | apple-code -p "add doc comments to this code" -l rust --tool-mode

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

# Edit a copy: --edit overwrites its input file
cp src/main.rs main.generated.rs
cargo run -- -p "Import anyhow::Context and add .context(\"Failed to load configuration\") to the Config::load result. Keep main returning anyhow::Result<()> and preserve the CLI behavior. Return the complete source file." -l rust --tool-mode -e main.generated.rs

# Review the result before integrating it; diff exits with 1 when files differ
# Do not replace src/main.rs until the generated code has been checked and tested
diff -u src/main.rs main.generated.rs

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

## OpenAI-compatible server

```bash
apple-code serve                            # 127.0.0.1:11434
apple-code serve --port 8080 --host 0.0.0.0  # trusted networks only
```

| Method | Path | Description |
| --- | --- | --- |
| GET | `/health` | Native model availability; 503 if unavailable |
| GET | `/v1/models` | Lists `apple-intelligence` |
| POST | `/v1/chat/completions` | Text chat, JSON or streaming SSE |

```bash
curl http://localhost:11434/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"apple-intelligence","messages":[{"role":"system","content":"Be concise."},{"role":"user","content":"Explain Rust ownership."}],"max_tokens":512}'

curl -N http://localhost:11434/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"apple-intelligence","messages":[{"role":"user","content":"Count to 5."}],"stream":true}'
```

```typescript
import OpenAI from "openai";
const client = new OpenAI({
  baseURL: "http://localhost:11434/v1",
  apiKey: "not-needed",
});
const response = await client.chat.completions.create({
  model: "apple-intelligence",
  messages: [{ role: "user", content: "Summarize the history of computing." }],
});
console.log(response.choices[0]?.message.content);
```

The server calls the Rust API of `apple_ai`; only that crate compiles and owns
the native Swift bridge. Initial `system` messages become native instructions;
`user`/`assistant` history uses a native transcript. Each request is independent
and must end with a user message, with alternating user/assistant turns after
initial system messages. No CLI code-generation instructions or mock fallback
are applied to HTTP requests.

Both HTTP response modes use the native text stream, buffered for non-streaming
responses. SSE sends role/content/stop chunks and `[DONE]` on success. Native
errors before the first delta produce an HTTP error; later failures produce an
SSE error event and close the stream without pretending success. Token usage is
omitted because this integration does not expose reliable tokenizer counts.

Supported options: `temperature` (0–2), `max_tokens` (1–8192, default 512;
`max_completion_tokens` is an alias), and `stream`. Tools, images, structured
output and unknown fields are rejected explicitly. Requests are limited to
256 KiB/256 messages and 120 seconds, with one active request (429 when busy).
Disconnecting the SSE client drops the native stream; cancellation is cooperative
at the next delta, not an immediate interruption of SDK inference.

**Security:** no authentication, TLS or CORS is configured. Loopback is the
default. Do not expose `0.0.0.0` on an untrusted network without an authenticated
TLS reverse proxy. Prompts are not logged normally; avoid debug logging with
sensitive content.

**Pinned dependency:** this checkout uses `apple_ai` 0.2.0 from its Git repository,
pinned to a tested commit in `Cargo.toml`. Cargo downloads it automatically; no
nested clone is required. The optional local `apple_ai/` checkout is ignored by
Git and excluded from the package. Publish the crate version on crates.io before
publishing/installing this updated application from crates.io.
The `server` feature is enabled by default; `--no-default-features` omits it.

Run the model-backed endpoint smoke test with:

```bash
cargo test --all-features native_backend_serves_health_json_and_sse -- --ignored
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
