# CLAUDE.md

## Project Overview

**handy-local-rules** is a lightweight local HTTP server and CLI tool written in Rust that provides an OpenAI-compatible Chat Completion API for text transformation. It's designed as a post-processing backend for the Handy application.

**Purpose:** Transform transcribed text using deterministic rules (e.g., "slash" → "/") without requiring a real LLM.

## Tech Stack

- **Language:** Rust (Edition 2024)
- **Web Framework:** axum 0.7
- **Async Runtime:** tokio
- **Serialization:** serde / serde_json
- **Regex:** regex crate
- **Glob:** glob crate (for pattern matching)
- **File Watching:** notify (for hot-reload)
- **Logging:** tracing / tracing-subscriber
- **CLI:** clap (with subcommands)
- **OpenAPI:** utoipa + utoipa-swagger-ui

## Project Structure

```
src/
├── main.rs              # Entry point, CLI parsing, subcommands
├── server.rs            # HTTP server setup, routes, Swagger
├── handlers.rs          # Request handlers
├── config.rs            # Configuration management
├── error.rs             # Error types
├── static/
│   └── index.html       # Dashboard UI
├── rules/
│   ├── mod.rs           # Rule engine module
│   ├── engine.rs        # Rule application logic
│   ├── loader.rs        # Rules file loading + hot-reload
│   └── types.rs         # Rule data structures
└── models/
    ├── mod.rs
    ├── request.rs       # OpenAI request types
    └── response.rs      # OpenAI response types
```

## CLI Usage

The tool supports both server and CLI modes via subcommands:

```bash
# Show help
handy-rules --help
handy-rules <command> --help

# Start HTTP server (default if no command given)
handy-rules serve
handy-rules serve --port 9000 --host 0.0.0.0

# Transform text directly (CLI mode)
handy-rules transform "foo slash bar"
echo "input" | handy-rules transform --stdin

# List all loaded rules
handy-rules list-rules

# Validate rules files
handy-rules validate
```

### Global Options

```bash
-c, --config <FILE>     # Path to config.json
-r, --rules <PATH>      # Additional rules file/glob/directory
-l, --log-level <LEVEL> # Log level (trace/debug/info/warn/error)
```

## Configuration

Configuration is loaded from (in order of precedence):

1. CLI arguments (highest priority)
2. Config file specified via `--config`
3. `config.json` in current directory (if exists)
4. `~/.handy-local-rules/config.json` (if exists)
5. Built-in defaults

### Default Config Directory

The application looks for configuration in `~/.handy-local-rules/`:

- `~/.handy-local-rules/config.json` — Main configuration file
- `~/.handy-local-rules/rules.json` — Rules file
- `~/.handy-local-rules/*.json` — Additional rules files (loaded as glob)

### config.json

```json
{
  "host": "127.0.0.1",
  "port": 61234,
  "rules_paths": ["rules.json", "custom-rules/*.json", "extra-rules/"],
  "api_key": null,
  "log_level": "info",
  "max_log_entries": 1000,
  "cors_enabled": true
}
```

### rules_paths

Accepts multiple formats:

- **Single file:** `"rules.json"`
- **Array of files:** `["rules.json", "custom.json"]`
- **Glob patterns:** `"rules/*.json"`
- **Directories:** `"rules/"` (loads all .json files)

## Commands

### Development

```bash
make setup              # Initial setup (install dependencies + git hooks)
make build              # Build debug version
make dev                # Development mode with hot-reload
cargo run -- serve      # Run server
```

### tmux Collaboration

When the user starts the dev server in a tmux session, Claude can access it:

**Current sessions:** `!tmux list-sessions 2>/dev/null || echo "none"`

```bash
# List all sessions
tmux list-sessions

# List windows in a session
tmux list-windows -t handy

# Read session output (last 20 lines)
tmux capture-pane -t handy -p | tail -20

# Read full scrollback buffer
tmux capture-pane -t handy -p -S -

# Send keys to session
tmux send-keys -t handy "command" Enter

# Kill session
tmux kill-session -t handy
```

### Code Quality

```bash
make fmt                # Format code (Rust + Markdown)
make lint               # Run clippy
make test               # Run tests
make check              # All checks (fmt + lint + test)
```

### Release

```bash
make release            # Build optimized release binary
```

## Code Conventions

### Rust Style

- Follow `rustfmt` configuration in `rustfmt.toml`
- All warnings treated as errors (`-D warnings`)
- Use `tracing` for logging, not `println!`
- Prefer `thiserror` for error types
- Use `?` operator for error propagation

### Git Workflow

**IMPORTANT: Commits require explicit user approval.**

- **NEVER** create commits automatically after completing tasks
- **ONLY** commit when the user explicitly requests it (e.g., "commit", "add and commit")
- If the commit instruction was not given in the immediately preceding message, use the `AskUserQuestion` tool to confirm before committing
- When in doubt, always ask first
- **NEVER** add `Co-Authored-By` or Claude Code attribution to commit messages

### Commit Messages

Follow Conventional Commits:

```
feat: add new feature
fix: bug fix
docs: documentation only
refactor: code refactoring
test: add/update tests
chore: maintenance
```

### API Design

- All endpoints return JSON
- Follow OpenAI API structure for `/v1/chat/completions`
- Use appropriate HTTP status codes
- Include request ID in responses

## Key Files

| File                  | Purpose                          |
| --------------------- | -------------------------------- |
| `config.json`         | Server configuration             |
| `config.example.json` | Example configuration            |
| `rules/`              | Transformation rules (see below) |
| `Cargo.toml`          | Rust dependencies                |
| `rustfmt.toml`        | Rust formatting config           |
| `clippy.toml`         | Clippy linter config             |

### Rules Directory

**IMPORTANT:** All rules must be stored in the `rules/` directory, organized by category.

```
rules/
├── de
│   └── satzzeichen.json
├── dev-kommandos
│   └── git.json
└── general
    └── cleanup.json
```

- `de/` — German language rules (Satzzeichen, etc.)
- `dev-kommandos/` — Developer commands (Git shortcuts, etc.)
- `general/` — General cleanup rules

## API Endpoints

| Method | Path                     | Description                  |
| ------ | ------------------------ | ---------------------------- |
| GET    | `/`                      | Dashboard UI                 |
| GET    | `/health`                | Health check                 |
| POST   | `/v1/chat/completions`   | Process text with rules      |
| GET    | `/v1/models`             | List available models        |
| GET    | `/v1/rules`              | List all loaded rules        |
| POST   | `/v1/rules/:id/toggle`   | Toggle rule enabled/disabled |
| GET    | `/v1/logs`               | Get transformation logs      |
| DELETE | `/v1/logs`               | Clear transformation logs    |
| GET    | `/swagger-ui/`           | Swagger UI                   |
| GET    | `/api-docs/openapi.json` | OpenAPI spec                 |

## Rule Types

### 1. Regex Rules (default)

Pattern-based text replacement using Rust regex:

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

Available functions:

- `uppercase` / `upper` — Convert to uppercase
- `lowercase` / `lower` — Convert to lowercase
- `trim` — Trim whitespace
- `trim_start` / `ltrim` — Trim leading whitespace
- `trim_end` / `rtrim` — Trim trailing whitespace
- `capitalize` — Capitalize first letter
- `reverse` — Reverse string
- `normalize_whitespace` — Multiple spaces → single space

### 3. Shell Rules

Execute shell commands (input via stdin, output via stdout):

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

## Hot-Reload

- **Rules files:** Automatically reloaded when modified (no restart needed)
- **Rust code:** Use `cargo watch -x 'run -- serve'` for development

## Dashboard UI

Access the web dashboard at `http://localhost:61234/`:

- View all loaded rules with toggle switches to enable/disable
- Test transformations interactively
- Monitor server health
- Links to Swagger UI and logs

## Transformation Logging

All transformations are logged and accessible via API:

```bash
# Get recent logs
curl http://localhost:61234/v1/logs

# Clear logs
curl -X DELETE http://localhost:61234/v1/logs
```

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Run specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Watch mode
cargo watch -x test

# With coverage (requires cargo-tarpaulin)
cargo tarpaulin
```

## Test Driven Development (TDD)

This project follows TDD principles. When implementing new features or fixing bugs:

### TDD Workflow

1. **Red** — Write a failing test first
2. **Green** — Write minimal code to make the test pass
3. **Refactor** — Clean up while keeping tests green

### Guidelines

- **Write tests BEFORE implementation code**
- Each new feature must have corresponding unit tests
- Bug fixes must include a regression test
- Run `cargo test` before committing

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = "test input";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, "expected output");
    }
}
```

### Test Naming Convention

- `test_<function>_<scenario>_<expected_behavior>`
- Example: `test_apply_regex_rule_with_word_boundary_replaces_only_whole_words`

### What to Test

| Component           | Test Focus                                      |
| ------------------- | ----------------------------------------------- |
| `rules/engine.rs`   | Rule application, priority ordering, edge cases |
| `rules/types.rs`    | Rule parsing, builtin functions                 |
| `models/request.rs` | Request parsing, content extraction             |
| `config.rs`         | Config loading, merging, defaults               |
| `handlers.rs`       | HTTP responses, error handling                  |

## Environment Variables

| Variable   | Default | Description                             |
| ---------- | ------- | --------------------------------------- |
| `RUST_LOG` | `info`  | Log level (trace/debug/info/warn/error) |
| `API_KEY`  | (none)  | Optional API key for authentication     |

## Performance Targets

- Response latency: ≤ 30ms
- Binary size: ≤ 5MB (release, stripped)
- Memory usage: ≤ 20MB at idle

## Debugging Transformations

When debugging rule issues, use the `logs` command to inspect input/output:

### View Recent Transformations

```bash
# Show last 10 transformations (default)
handy-rules logs

# Show last 20 transformations
handy-rules logs -n 20

# Follow mode: continuously print new transformations
handy-rules logs -f

# Show and clear logs
handy-rules logs --clear
```

### Clear Logs

```bash
handy-rules logs --clear
# Or via API:
curl -s -X DELETE http://localhost:61234/v1/logs
```

### Check Specific Rule

```bash
curl -s http://localhost:61234/v1/rules | grep -o '"rule-id"[^}]*}'
```

### Test Single Transformation

```bash
curl -s -X POST http://localhost:61234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Test input here"}' | jq -r '.choices[0].message.content'
```
