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
apple-code serve --port 8080 --logs           # request and generation diagnostics
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
`user`/`assistant` history uses a native transcript. Each request is independent.
Consecutive messages of the same role are accepted and preserved in order.
Assistant placeholders with `content: null` (or omitted content) and no tool
calls are ignored before native generation. User and system messages still
require non-empty text; tool-call fields and non-text content remain unsupported.
Requests ending with an assistant message retain the history and use the native
bridge's `Continue.` prompt. System messages must precede the conversation.
No CLI code-generation instructions or mock fallback are applied to HTTP requests.

Tool calling is not supported. Clients may send `tools: []` and
`tool_choice: "none"`; non-empty tool lists and other tool choices are rejected.
For OpenCode V2, declare `capabilities.tools: false` and set the model's `body`
to `{"tools": [], "tool_choice": "none"}` to force text-only requests.

Both HTTP response modes use the native text stream, buffered for non-streaming
responses. SSE sends role/content/stop chunks and `[DONE]` on success. Native
errors before the first delta produce an HTTP error; later failures produce an
SSE error event and close the stream without pretending success. Token usage is
omitted because this integration does not expose reliable tokenizer counts.

Supported options: `temperature` (0–2), `max_tokens` (1–8192, default 512;
`max_completion_tokens` is an alias), and `stream`. The optional string
`reasoning_effort` is accepted for client compatibility but ignored: the native
API exposes no reasoning-effort control. Non-empty tool lists, images, structured
output and unknown fields are rejected explicitly. Requests are limited to
256 KiB/256 messages and 120 seconds, with one active request (429 when busy).
Disconnecting the SSE client drops the native stream; cancellation is cooperative
at the next delta, not an immediate interruption of SDK inference.

Use `serve --logs` (alias of `--verbose`) to enable server diagnostics on stderr.
Each HTTP response includes a server-generated `x-request-id`. Logs report that
ID, method, route, HTTP status, time to response headers, message roles/counts,
input/system byte sizes, requested output limit, generation duration and output
byte count. Streaming completion, disconnection and errors are logged separately
from HTTP headers, since HTTP 200 does not mean the stream finished successfully.
Prompts, generated text, query strings and request headers are not included in
these diagnostics. Byte sizes are not token counts. Without `--logs`, routine
request logs are disabled; warnings and errors still use the existing logger.

**Security:** no authentication, TLS or CORS is configured. Loopback is the
default. Do not expose `0.0.0.0` on an untrusted network without an authenticated
TLS reverse proxy. Prompts are not logged normally; avoid debug logging with
sensitive content.

**Registry dependency:** this checkout uses [`apple_ai` 0.2.0](https://crates.io/crates/apple_ai/0.2.0)
from crates.io. Cargo downloads it automatically; no Git dependency or nested
clone is required. The optional local `apple_ai/` checkout is ignored by Git and
excluded from the package. The server compatibility fixes and request logs
described here are included in `apple-code-assistant` 0.1.4. Updating `apple_ai`
alone does not update an installed `apple-code` binary; reinstall or upgrade
`apple-code-assistant` to get these changes.
The `server` feature is enabled by default; `--no-default-features` omits it.

Run the model-backed endpoint smoke test with:

```bash
cargo test --all-features native_backend_serves_health_json_and_sse -- --ignored
```

### Context limits and OpenCode troubleshooting

The on-device Foundation Models system model supports a **4,096-token context
window**, shared by system instructions, conversation history, the current
message and generated output. See Apple's documentation on
[context size limits](https://developer.apple.com/documentation/foundationmodels/generating-content-and-performing-tasks-with-foundation-models).
This is a model limit, not a server setting:

- Increasing OpenCode's declared `limit.context` does not enlarge the native
  model's context window.
- `max_tokens` limits generated output, not input. The HTTP API's accepted range
  of 1–8192 is parameter validation, not a guarantee that the model can generate
  that many tokens or that the complete request fits.
- Declared input limits and automatic compaction are client-side budgeting aids;
  they do not guarantee that injected instructions fit the native tokenizer.
- The HTTP server does not silently truncate prompts or discard history.

#### Why even a short greeting can fail

OpenCode may add agent instructions, extension guidance, tool descriptions,
`AGENTS.md` files and conversation history before sending your message. During
local testing, a request containing a short greeting included **41–45 KB of
system instructions alone**. That request exceeded the native context window.
These sizes are an observed example, not a general byte-to-token conversion.

Use `apple-code serve --port 8080 --logs` to inspect `system_bytes`, `input_bytes`,
message roles and the requested output limit without logging message content.
If system instructions already exceed the context budget, shortening your
question or compacting only the conversation history will not solve the problem.
OpenCode may report:

```text
The compaction request cannot be reduced further without losing the latest exchange or checkpoint
```

Retrying the same oversized request does not make it fit. Stop pending attempts
before testing again. `400 context_length_exceeded` identifies an oversized
native context; `429 server_busy` means another generation is already running
and may occur when attempts overlap.

#### Load the project's minimal OpenCode profile

The `opencode.json` in this checkout defines the text-only `apple-chat` agent,
short instructions, model budgets and project-scoped extension restrictions.
Your global `~/.config/opencode/opencode.json` is separate. OpenCode searches
configuration upward from its working directory: launching from a parent folder
such as `Downloads` does **not** load `Downloads/apple-code/opencode.json`.

From the checkout root, start a new, isolated command-line session:

```bash
opencode --standalone run --agent apple-chat --model apple/apple-intelligence 'Bonjour'
```

For the interactive interface, launch from the checkout root with
`opencode --standalone`, or pass the checkout's absolute path as the directory
argument. Create a **new conversation** and select `apple-chat` with
`apple/apple-intelligence`; an existing session may retain its previous agent
and oversized history. `--standalone` isolates the OpenCode server process; it
neither enlarges the Apple model's context nor disables extensions by itself.

This profile is for short text conversations, not a full coding agent: the HTTP
integration does not support tool calls, file editing or shell execution.

#### Work within the budget

- Keep system instructions short and remove irrelevant injected guidance. Do not
  remove important security or task constraints merely to make a request fit.
- Send a relevant function, excerpt or diff instead of an entire file or repo.
- Process long documents in independent chunks, then combine their results.
- For longer conversations, retain a concise summary of goals, decisions and
  essential facts, plus recent turns; reserve space for the next response.
- If the task requires extensive instructions, tools or large files at once,
  choose a model/provider with a verified larger context window. Sending data
  to a remote provider is a separate privacy and cost decision.

Automatic server-side truncation is not a safe substitute: it can silently drop
requirements or evidence and change the meaning of the request.

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
