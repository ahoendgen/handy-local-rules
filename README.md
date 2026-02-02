# Handy Local Rules

A lightweight, local HTTP server and CLI tool that provides an OpenAI-compatible Chat Completion API for deterministic text transformation. Designed as a post-processing backend for [Handy](https://github.com/handy-app) and similar applications.

## Features

- **OpenAI-Compatible API** — Drop-in replacement for post-processing workflows
- **Multiple Rule Types** — Regex patterns, built-in functions, and shell commands
- **Hot-Reload** — Rules are automatically reloaded when files change
- **CLI Mode** — Transform text directly from the command line
- **Dashboard UI** — Web interface to view rules and test transformations
- **Swagger UI** — Interactive API documentation
- **Multiple Rule Sources** — Load rules from files, directories, or glob patterns
- **Transformation Logging** — Track all transformations via API

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourname/handy-local-rules.git
cd handy-local-rules

# Build release binary
cargo build --release

# Binary is at target/release/handy-rules
```

### Pre-built Binaries

Download from the [Releases](https://github.com/yourname/handy-local-rules/releases) page.

## Quick Start

### 1. Create a rules file

Create `rules.json`:

```json
[
  {
    "id": "slash",
    "description": "spoken 'slash' -> /",
    "type": "regex",
    "pattern": "(?i)\\bslash\\b",
    "replacement": "/",
    "priority": 100,
    "enabled": true
  },
  {
    "id": "dot",
    "description": "spoken 'dot' -> .",
    "type": "regex",
    "pattern": "(?i)\\bdot\\b",
    "replacement": ".",
    "priority": 80,
    "enabled": true
  }
]
```

### 2. Start the server

```bash
handy-rules serve
```

### 3. Test it

```bash
curl -X POST http://localhost:61234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"foo slash bar dot com"}]}'
```

**Response:**

```json
{
  "id": "local-abc123...",
  "object": "chat.completion",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "foo / bar . com"
    }
  }]
}
```

## Usage

### Server Mode

```bash
# Start with defaults (localhost:61234)
handy-rules serve

# Custom host and port
handy-rules serve --host 0.0.0.0 --port 9000

# With custom config
handy-rules serve --config my-config.json

# With additional rules
handy-rules serve --rules extra-rules.json
```

### CLI Mode

```bash
# Transform text directly
handy-rules transform "foo slash bar"
# Output: foo / bar

# Read from stdin
echo "hello dot world" | handy-rules transform --stdin
# Output: hello . world

# List all rules
handy-rules list-rules

# Validate rules files
handy-rules validate
```

### Global Options

```bash
-c, --config <FILE>     Path to config.json
-r, --rules <PATH>      Additional rules file/glob/directory
-l, --log-level <LEVEL> Log level (trace/debug/info/warn/error)
-h, --help              Show help
-V, --version           Show version
```

## Configuration

Configuration is loaded from the following locations (in order of precedence):

1. CLI arguments (highest priority)
2. Config file specified via `--config`
3. `config.json` in current directory
4. `~/.handy-local-rules/config.json`
5. Built-in defaults

### Default Config Directory

The application automatically looks for configuration in `~/.handy-local-rules/`:

```
~/.handy-local-rules/
├── config.json      # Main configuration
├── rules.json       # Default rules file
└── *.json           # Additional rule files (loaded via glob)
```

### config.json

```json
{
  "host": "127.0.0.1",
  "port": 61234,
  "rules_paths": [
    "rules.json",
    "custom-rules/*.json",
    "extra-rules/"
  ],
  "api_key": null,
  "log_level": "info",
  "max_log_entries": 1000,
  "cors_enabled": true
}
```

### Configuration Options

| Option               | Default       | Description                                         |
| -------------------- | ------------- | --------------------------------------------------- |
| `host`               | `127.0.0.1`   | Host/IP to bind to (`0.0.0.0` for all interfaces)   |
| `port`               | `61234`       | Port to listen on (uses private port range)         |
| `rules_paths`        | `rules.json`  | Rules file(s), directories, or glob patterns        |
| `api_key`            | `null`        | Optional API key for authentication                 |
| `log_level`          | `info`        | Log level: `trace`, `debug`, `info`, `warn`, `error`|
| `max_log_entries`    | `1000`        | Maximum transformation log entries to keep in memory|
| `cors_enabled`       | `true`        | Enable CORS headers for cross-origin requests       |
| `enable_shell_rules` | `false`       | Enable shell command rules (**security risk**)      |

### Port Selection

The default port `61234` is in the private/dynamic port range (49152-65535), which minimizes conflicts with other services. The server checks if the port is available at startup and provides helpful error messages if it's in use.

### Rules Paths

The `rules_paths` option accepts multiple formats:

```json
// Single file
"rules_paths": "rules.json"

// Multiple files
"rules_paths": ["rules.json", "custom.json"]

// Glob pattern
"rules_paths": "rules/*.json"

// Directory (loads all .json files)
"rules_paths": "rules/"

// Mixed
"rules_paths": [
  "rules.json",
  "custom-rules/*.json",
  "extra/"
]
```

## Rule Types

### 1. Regex Rules

Pattern-based text replacement using Rust regex syntax:

```json
{
  "id": "slash",
  "type": "regex",
  "pattern": "(?i)\\bslash\\b",
  "replacement": "/",
  "priority": 100,
  "enabled": true
}
```

**Options:**

- `pattern` — Rust regex pattern
- `replacement` — Replacement string (supports `$1`, `$2` backreferences)
- `priority` — Higher priority rules are applied first
- `enabled` — Set to `false` to disable without removing
- `ignore_case` — Case-insensitive matching (or use `(?i)` in pattern)
- `stop_on_match` — Stop processing further rules if this rule matches

**Common Patterns:**

| Pattern | Description |
|---------|-------------|
| `(?i)\\bword\\b` | Case-insensitive whole word |
| `\\s+` | Multiple whitespace |
| `^prefix` | Start of string |
| `suffix$` | End of string |

### 2. Function Rules

Built-in transformation functions:

```json
{
  "id": "normalize",
  "type": "function",
  "pattern": "normalize_whitespace",
  "priority": 10,
  "enabled": true
}
```

**Available Functions:**

| Function              | Description                        |
| --------------------- | ---------------------------------- |
| `uppercase` / `upper` | Convert to uppercase               |
| `lowercase` / `lower` | Convert to lowercase               |
| `trim`                | Remove leading/trailing whitespace |
| `trim_start` / `ltrim`| Remove leading whitespace          |
| `trim_end` / `rtrim`  | Remove trailing whitespace         |
| `capitalize`          | Capitalize first letter            |
| `reverse`             | Reverse the string                 |
| `normalize_whitespace`| Multiple spaces → single space     |

### 3. Shell Rules

Execute shell commands for complex transformations:

```json
{
  "id": "custom-transform",
  "type": "shell",
  "pattern": "cat | sed 's/foo/bar/g'",
  "timeout_ms": 5000,
  "priority": 50,
  "enabled": true
}
```

**⚠️ Security Warning:** Shell rules can execute arbitrary commands on your system. They are **disabled by default**. To enable, set `enable_shell_rules: true` in your config.

**How it works:**

1. Input text is sent to the command via **stdin**
2. Command output from **stdout** becomes the result
3. Commands run with a configurable timeout (default: 5000ms)

**Examples:**

```json
// Uppercase via tr
{"pattern": "cat | tr 'a-z' 'A-Z'"}

// Custom sed replacement
{"pattern": "cat | sed 's/foo/bar/g'"}

// Python script
{"pattern": "python3 transform.py"}
```

## API Endpoints

| Method | Path                        | Description                   |
| ------ | --------------------------- | ----------------------------- |
| GET    | `/`                         | Dashboard UI                  |
| GET    | `/health`                   | Health check                  |
| POST   | `/v1/chat/completions`      | Process text with rules       |
| GET    | `/v1/models`                | List available models         |
| GET    | `/v1/rules`                 | List all loaded rules         |
| POST   | `/v1/rules/{id}/toggle`     | Toggle rule enabled/disabled  |
| GET    | `/v1/logs`                  | Get transformation logs       |
| DELETE | `/v1/logs`                  | Clear transformation logs     |
| GET    | `/swagger-ui/`              | Swagger UI                    |
| GET    | `/api-docs/openapi.json`    | OpenAPI specification         |

### Chat Completion Request

```bash
curl -X POST http://localhost:61234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "local-rules",
    "messages": [
      {"role": "user", "content": "foo slash bar dot com"}
    ]
  }'
```

The server accepts multiple input formats:

- `messages` array (OpenAI format)
- `prompt` field
- `input` field
- `text` field

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

## Integration with Handy

1. Start the local rules server:
   ```bash
   handy-rules serve
   ```

2. In Handy, configure a post-processing provider:
   - **Base URL:** `http://127.0.0.1:61234`
   - **API Key:** (leave empty or set if configured)

3. Enable post-processing in Handy settings

## Web Dashboard

Access the dashboard at `http://localhost:61234/`:

- View all loaded rules with status and priority
- **Toggle rules on/off** with instant persistence to rules files
- Test transformations interactively
- Monitor server health
- Quick links to Swagger UI and API docs

## Hot-Reload

Rules files are automatically watched for changes. When you modify a rules file, the server reloads it without requiring a restart.

For development with Rust code changes, use:

```bash
cargo watch -x 'run -- serve'
```

**Note:** When toggling rules via the dashboard or API, the changes are persisted back to the rules file. This will reformat the JSON and **remove any comments** (JSON does not support comments).

## Example Rules

### Common Symbols

```json
[
  {"id": "slash", "pattern": "(?i)\\bslash\\b", "replacement": "/", "priority": 100},
  {"id": "backslash", "pattern": "(?i)\\bbackslash\\b", "replacement": "\\\\", "priority": 90},
  {"id": "dot", "pattern": "(?i)\\bdot\\b", "replacement": ".", "priority": 80},
  {"id": "at-sign", "pattern": "(?i)\\bat sign\\b", "replacement": "@", "priority": 70},
  {"id": "underscore", "pattern": "(?i)\\bunderscore\\b", "replacement": "_", "priority": 60},
  {"id": "dash", "pattern": "(?i)\\bdash\\b", "replacement": "-", "priority": 50}
]
```

### Text Cleanup

```json
[
  {"id": "normalize", "type": "function", "pattern": "normalize_whitespace", "priority": 10},
  {"id": "trim", "type": "function", "pattern": "trim", "priority": 5}
]
```

### Programming Symbols

```json
[
  {"id": "open-paren", "pattern": "(?i)\\bopen paren\\b", "replacement": "(", "priority": 100},
  {"id": "close-paren", "pattern": "(?i)\\bclose paren\\b", "replacement": ")", "priority": 100},
  {"id": "open-bracket", "pattern": "(?i)\\bopen bracket\\b", "replacement": "[", "priority": 100},
  {"id": "close-bracket", "pattern": "(?i)\\bclose bracket\\b", "replacement": "]", "priority": 100},
  {"id": "equals", "pattern": "(?i)\\bequals\\b", "replacement": "=", "priority": 100},
  {"id": "plus", "pattern": "(?i)\\bplus\\b", "replacement": "+", "priority": 100}
]
```

## Development

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs))
- Node.js 18+ (for git hooks)

### Setup

```bash
# Clone and setup
git clone https://github.com/yourname/handy-local-rules.git
cd handy-local-rules
make setup

# Run in development
cargo watch -x 'run -- serve'

# Run tests
cargo test

# Format and lint
make check
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
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure `make check` passes
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the need for local, offline text transformation
- Built with [axum](https://github.com/tokio-rs/axum) and [tokio](https://tokio.rs)
- OpenAPI documentation powered by [utoipa](https://github.com/juhaku/utoipa)
