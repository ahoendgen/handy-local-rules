# Handy Local Rules

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![OpenAI Compatible](https://img.shields.io/badge/OpenAI-Compatible-green.svg)](https://platform.openai.com/docs/api-reference/chat)

A lightweight, local HTTP server and CLI tool that provides an OpenAI-compatible Chat Completion API for deterministic text transformations.

## Why This Exists

[Handy](https://handy.computer/) ([GitHub](https://github.com/cjpais/Handy)) is a voice-to-text application that supports post-processing transcribed text via OpenAI-compatible APIs. While you can use cloud LLMs for this, simple text transformations like converting spoken "period" to "." don't require an expensive language model.

**handy-local-rules** provides a fast, offline, and free alternative. It processes text using deterministic regex rules—perfect for:

- Converting spoken punctuation ("period", "comma", "new line") to symbols
- Fixing common transcription patterns
- Custom text normalization
- Any application that accepts OpenAI-compatible post-processing

## Features

- **OpenAI-Compatible API**: Drop-in replacement for post-processing workflows
- **Rule Engine**:
  - **Regex**: Powerful pattern-based replacements
  - **Functions**: Built-in functions like `trim`, `uppercase`, `normalize_whitespace`
  - **Shell**: Execute external scripts (optional, security flag required)
- **Hot-Reload**: Rules are automatically reloaded when files change
- **Web Dashboard**: Built-in UI for testing rules and monitoring status
- **CLI Mode**: Transform text directly from the terminal
- **Swagger UI**: Interactive API documentation
- **macOS Service**: Auto-start via launchd

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.85 or higher)
- Make (optional, for convenience commands)

### Build from Source

```bash
git clone https://github.com/ahoendgen/handy-local-rules.git
cd handy-local-rules
cargo build --release

# Binary is at target/release/handy-rules
./target/release/handy-rules --help
```

### Setup (Install Rules)

```bash
# Copy rules and config to ~/.handy-local-rules/
handy-rules setup

# Overwrite existing files
handy-rules setup --force
```

### macOS Service Installation

```bash
# Install (builds release, installs service + CLI)
make install

# Uninstall
make uninstall

# Reinstall (after code changes)
make reinstall

# Control service
make service-start
make service-stop
make service-status
make service-logs
```

After `make install`:

- CLI available at `/usr/local/bin/handy-rules`
- Service auto-starts on login
- Logs at `~/.handy-local-rules/handy-local-rules.log`

## Quick Start

1. **Run setup**

   ```bash
   handy-rules setup
   ```

2. **Start the server**

   ```bash
   handy-rules serve
   ```

3. **Check status**

   ```bash
   handy-rules status
   ```

   Output:

   ```
   handy-local-rules Status

   Service installed: ✓ yes
   Service running:   ✓ yes
   CLI installed:     ✓ yes
   Health check:      ✓ ok (http://127.0.0.1:61234/health)

   Configuration:
     Config dir: /Users/you/.handy-local-rules
     Server:     127.0.0.1:61234
   ```

## API Usage

The server exposes an OpenAI-compatible `/v1/chat/completions` endpoint.

### Chat Completion (OpenAI Format)

```bash
curl -X POST http://localhost:61234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local-rules",
    "messages": [
      {"role": "user", "content": "hello world period new line how are you question mark"}
    ]
  }'
```

Response:

```json
{
  "id": "local-abc123",
  "object": "chat.completion",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "hello world.\nhow are you?"
      }
    }
  ]
}
```

### Alternative Input Formats

The server also accepts `prompt`, `input`, or `text` fields:

```bash
# Using prompt field
curl -X POST http://localhost:61234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"prompt": "test slash example"}'

# Using input field
curl -X POST http://localhost:61234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"input": "test slash example"}'
```

### Health Check

```bash
curl http://localhost:61234/health
```

```json
{
  "status": "ok",
  "version": "0.1.0",
  "rules_loaded": 12
}
```

### List Models

```bash
curl http://localhost:61234/v1/models
```

### API Endpoints

| Method | Path                     | Description             |
| ------ | ------------------------ | ----------------------- |
| GET    | `/`                      | Dashboard UI            |
| GET    | `/health`                | Health check            |
| POST   | `/v1/chat/completions`   | Transform text          |
| GET    | `/v1/models`             | List available models   |
| GET    | `/v1/rules`              | List all rules          |
| POST   | `/v1/rules/{id}/toggle`  | Toggle rule on/off      |
| GET    | `/v1/logs`               | Get transformation logs |
| DELETE | `/v1/logs`               | Clear logs              |
| GET    | `/swagger-ui/`           | Swagger UI              |
| GET    | `/api-docs/openapi.json` | OpenAPI spec            |

## Configuration

Configuration is loaded from (in order of precedence):

1. CLI arguments
2. Config file via `--config`
3. `./config.json`
4. `~/.handy-local-rules/config.json`

### Example `config.json`

```json
{
  "host": "127.0.0.1",
  "port": 61234,
  "rules_paths": ["rules/**/*.json"],
  "log_level": "info",
  "enable_shell_rules": false
}
```

### Options

| Option               | Default      | Description                                 |
| -------------------- | ------------ | ------------------------------------------- |
| `host`               | `127.0.0.1`  | Host/IP (`0.0.0.0` for all interfaces)      |
| `port`               | `61234`      | Port (private port range)                   |
| `rules_paths`        | `rules.json` | Rule files, directories, or glob patterns   |
| `log_level`          | `info`       | Log level: `trace`, `debug`, `info`, `warn` |
| `enable_shell_rules` | `false`      | Enable shell rules (**security risk**)      |

## Defining Rules

Rules are defined in JSON files.

### Regex Rules (Default)

```json
{
  "id": "period",
  "description": "period -> .",
  "type": "regex",
  "pattern": "(?i)\\bperiod\\b",
  "replacement": ".",
  "priority": 100,
  "enabled": true
}
```

**Options:**

- `pattern` — Rust regex pattern
- `replacement` — Replacement text (supports `$1`, `$2` backreferences)
- `priority` — Higher priority = applied first
- `enabled` — Set to `false` to disable
- `ignore_case` — Case-insensitive matching
- `stop_on_match` — Stop processing after this rule matches

### Function Rules

```json
{
  "id": "cleanup",
  "type": "function",
  "pattern": "normalize_whitespace",
  "priority": 10
}
```

Available functions:

| Function               | Description                    |
| ---------------------- | ------------------------------ |
| `uppercase` / `upper`  | Convert to uppercase           |
| `lowercase` / `lower`  | Convert to lowercase           |
| `trim`                 | Remove leading/trailing spaces |
| `trim_start` / `ltrim` | Remove leading whitespace      |
| `trim_end` / `rtrim`   | Remove trailing whitespace     |
| `capitalize`           | Capitalize first letter        |
| `reverse`              | Reverse the string             |
| `normalize_whitespace` | Multiple spaces → single space |

### Shell Rules

```json
{
  "id": "custom-script",
  "type": "shell",
  "pattern": "python3 my_script.py",
  "timeout_ms": 5000,
  "priority": 50
}
```

**Note:** Requires `enable_shell_rules: true` in config.

## CLI Usage

```bash
# Start server
handy-rules serve
handy-rules serve --port 9000 --host 0.0.0.0

# Transform text
handy-rules transform "hello period world"
echo "test slash example" | handy-rules transform --stdin

# List rules
handy-rules list-rules

# Validate rules
handy-rules validate

# Check status
handy-rules status

# Open dashboard in browser
handy-rules dashboard
handy-rules dashboard --browser firefox

# View transformation logs
handy-rules logs
handy-rules logs -n 20      # Show last 20
handy-rules logs -f         # Follow mode
handy-rules logs --clear    # Show and clear

# Setup (copy rules)
handy-rules setup
handy-rules setup --force
```

### Global Options

```bash
-c, --config <FILE>     Path to config.json
-r, --rules <PATH>      Additional rules file/glob/directory
-l, --log-level <LEVEL> Log level (trace/debug/info/warn/error)
-h, --help              Show help
-V, --version           Show version
```

## Development

```bash
make setup      # Install git hooks & tools
make dev        # Server with hot-reload
make dev-tmux   # In tmux session with debug logging
make check      # Format, lint, and test
make test       # Run unit tests
make release    # Build release binary
```

### Project Structure

```
src/
├── main.rs          # CLI entry point
├── server.rs        # HTTP server
├── handlers.rs      # Request handlers
├── config.rs        # Configuration
├── error.rs         # Error types
├── static/          # Dashboard UI
├── rules/           # Rule engine
└── models/          # API types

rules/
├── de/              # German rules
│   └── satzzeichen.json
├── general/         # General rules
│   └── cleanup.json
└── dev-kommandos/   # Developer shortcuts
    └── git.json
```

## License

MIT License - See [LICENSE](LICENSE) file.
