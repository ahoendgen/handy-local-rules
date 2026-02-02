# Product Requirements Document (PRD)
## Local LLM-Compatible Rule Server for Handy — Rust Implementation

**Version:** 2.0
**Date:** 2026-02-02

---

## 1. Summary / Objective

Build a lightweight local HTTP server in **Rust** that:
- Exposes a minimal OpenAI-like Chat Completion API (the endpoint Handy uses for post-processing)
- Internally runs a deterministic rule engine (regex replacements, optional fuzzy matching)
- Transforms transcribed text according to user-defined rules

**Purpose:** Allow users to run post-processing rules (e.g., "slash" → "/") locally and offline, without modifying Handy.

---

## 2. Goals

- Expose a compatible `/v1/chat/completions` API for Handy
- Provide immediate, deterministic text replacements via regex rules
- Optional fuzzy matching for custom words
- Hot-reload rules without server restart
- Single binary distribution (no runtime dependencies)
- Cross-platform: Linux, macOS, Windows

---

## 3. Non-Goals

- Not a full LLM or model inference engine
- No modifications to Handy source code required
- Not a replacement for server-side LLM providers (except for post-processing)

---

## 4. Technology Stack

### Core Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.7.x | Async web framework |
| `tokio` | 1.x | Async runtime |
| `serde` | 1.x | Serialization framework |
| `serde_json` | 1.x | JSON parsing/generation |
| `regex` | 1.x | Regular expression engine |
| `notify` | 6.x | File system watcher for hot-reload |
| `tracing` | 0.1.x | Structured logging |
| `tracing-subscriber` | 0.3.x | Log output formatting |
| `uuid` | 1.x | Generate unique IDs for responses |
| `clap` | 4.x | CLI argument parsing |

### Optional Dependencies

| Crate | Purpose |
|-------|---------|
| `strsim` | Fuzzy string matching (Levenshtein, Jaro-Winkler) |
| `tower-http` | CORS, tracing middleware |

---

## 5. Project Structure

```
handy-local-rules/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── rules.json                    # Default rules file
├── config.json                   # Optional server config
├── scripts/
│   ├── start_local_llm.sh       # Linux/macOS launcher
│   └── start_local_llm.ps1      # Windows launcher
├── systemd/
│   └── handy-rules.service      # systemd unit file
├── src/
│   ├── main.rs                  # Entry point, CLI parsing
│   ├── server.rs                # HTTP server setup, routes
│   ├── handlers.rs              # Request handlers
│   ├── rules/
│   │   ├── mod.rs               # Rule engine module
│   │   ├── engine.rs            # Rule application logic
│   │   ├── loader.rs            # Rules file loading + hot-reload
│   │   └── types.rs             # Rule data structures
│   ├── models/
│   │   ├── mod.rs
│   │   ├── request.rs           # OpenAI request types
│   │   └── response.rs          # OpenAI response types
│   ├── config.rs                # Configuration management
│   └── error.rs                 # Error types
└── tests/
    ├── integration_tests.rs     # API integration tests
    └── rule_tests.rs            # Rule engine unit tests
```

---

## 6. Cargo.toml

```toml
[package]
name = "handy-local-rules"
version = "0.1.0"
edition = "2024"
authors = ["Your Name"]
description = "Local rule-based text transformer with OpenAI-compatible API"
license = "MIT"
repository = "https://github.com/yourname/handy-local-rules"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
notify = "6"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4"] }
clap = { version = "4", features = ["derive"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Optional fuzzy matching
strsim = { version = "0.11", optional = true }

[features]
default = []
fuzzy = ["strsim"]

[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"

[[bin]]
name = "handy-rules"
path = "src/main.rs"
```

---

## 7. Data Structures

### 7.1 Rule Definition

```rust
// src/rules/types.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub pattern: String,
    pub replacement: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub ignore_case: bool,
    #[serde(default)]
    pub fuzzy_key: bool,
}

fn default_priority() -> i32 { 0 }
fn default_enabled() -> bool { true }
```

### 7.2 OpenAI-Compatible Request

```rust
// src/models/request.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub messages: Option<Vec<Message>>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
```

### 7.3 OpenAI-Compatible Response

```rust
// src/models/response.rs

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

---

## 8. API Specification

### 8.1 Health Check

```
GET /health

Response 200:
{
  "status": "ok",
  "version": "0.1.0",
  "rules_loaded": 15
}
```

### 8.2 Chat Completion

```
POST /v1/chat/completions

Request:
{
  "model": "local-rules",
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "foo slash bar dot com"}
  ]
}

Response 200:
{
  "id": "local-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "object": "chat.completion",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "foo / bar . com"
      }
    }
  ],
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0
  }
}
```

### 8.3 Models (Optional)

```
GET /v1/models

Response 200:
{
  "data": [
    {"id": "local-rules", "object": "model"}
  ]
}
```

---

## 9. Core Implementation

### 9.1 Main Entry Point

```rust
// src/main.rs

use clap::Parser;

#[derive(Parser)]
#[command(name = "handy-rules")]
#[command(about = "Local rule-based text transformer")]
struct Args {
    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Path to rules.json
    #[arg(short, long, default_value = "rules.json")]
    rules: String,

    /// API key (optional)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // Initialize logging, load rules, start server
    // ...
}
```

### 9.2 Rule Engine Algorithm

```rust
// src/rules/engine.rs

impl RuleEngine {
    pub fn apply(&self, text: &str) -> String {
        let rules = self.rules.read().unwrap();

        // Sort by priority (descending)
        let mut sorted_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.enabled)
            .collect();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut result = text.to_string();

        for rule in sorted_rules {
            let regex = self.get_compiled_regex(rule)?;
            result = regex.replace_all(&result, &rule.replacement).to_string();
        }

        result
    }
}
```

### 9.3 Hot-Reload with notify

```rust
// src/rules/loader.rs

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

pub fn watch_rules_file(path: &Path, rules: Arc<RwLock<Vec<Rule>>>) {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv() {
            if let Ok(Event { kind: EventKind::Modify(_), .. }) = event {
                if let Ok(new_rules) = load_rules_from_file(path) {
                    let mut rules_guard = rules.write().unwrap();
                    *rules_guard = new_rules;
                    tracing::info!("Rules reloaded: {} rules active", rules_guard.len());
                }
            }
        }
    });
}
```

---

## 10. Configuration

### 10.1 config.json (Optional)

```json
{
  "host": "127.0.0.1",
  "port": 8080,
  "rules_path": "rules.json",
  "api_key": null,
  "log_level": "info",
  "enable_fuzzy": false,
  "fuzzy_threshold": 0.8
}
```

### 10.2 rules.json (Starter)

```json
[
  {
    "id": "slash",
    "description": "spoken slash -> /",
    "pattern": "(?i)\\bslash\\b",
    "replacement": "/",
    "priority": 100,
    "enabled": true,
    "ignore_case": true
  },
  {
    "id": "backslash",
    "description": "spoken backslash -> \\",
    "pattern": "(?i)\\bbackslash\\b",
    "replacement": "\\\\",
    "priority": 90,
    "enabled": true,
    "ignore_case": true
  },
  {
    "id": "dot",
    "description": "dot -> .",
    "pattern": "(?i)\\bdot\\b",
    "replacement": ".",
    "priority": 80,
    "enabled": true,
    "ignore_case": true
  },
  {
    "id": "at",
    "description": "at sign -> @",
    "pattern": "(?i)\\bat sign\\b",
    "replacement": "@",
    "priority": 70,
    "enabled": true,
    "ignore_case": true
  },
  {
    "id": "underscore",
    "description": "underscore -> _",
    "pattern": "(?i)\\bunderscore\\b",
    "replacement": "_",
    "priority": 60,
    "enabled": true,
    "ignore_case": true
  }
]
```

---

## 11. Build & Release

### 11.1 Development Build

```bash
cargo build
cargo run -- --port 8080 --rules rules.json
```

### 11.2 Release Build

```bash
cargo build --release
# Binary at: target/release/handy-rules
```

### 11.3 Cross-Compilation

Using `cross` for easy cross-compilation:

```bash
cargo install cross

# Linux (x86_64)
cross build --release --target x86_64-unknown-linux-gnu

# Linux (ARM64)
cross build --release --target aarch64-unknown-linux-gnu

# Windows
cross build --release --target x86_64-pc-windows-gnu

# macOS (requires macOS host or osxcross)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### 11.4 Release Artifacts

For each release, produce:
- `handy-rules-linux-x86_64.tar.gz`
- `handy-rules-linux-arm64.tar.gz`
- `handy-rules-macos-x86_64.tar.gz`
- `handy-rules-macos-arm64.tar.gz`
- `handy-rules-windows-x86_64.zip`

Each archive contains:
- Binary (`handy-rules` or `handy-rules.exe`)
- `rules.json` (starter)
- `README.md`
- Platform-specific launcher script

---

## 12. Launcher Scripts

### 12.1 Linux/macOS (start_local_llm.sh)

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVER_BIN="${SCRIPT_DIR}/handy-rules"
PORT="${PORT:-8080}"
HOST="${HOST:-127.0.0.1}"
RULES="${RULES:-${SCRIPT_DIR}/rules.json}"
HEALTH_URL="http://${HOST}:${PORT}/health"
TIMEOUT=5

# Check if already running
if curl -s --max-time 1 "${HEALTH_URL}" > /dev/null 2>&1; then
    echo "Server already running at ${HEALTH_URL}"
    exit 0
fi

# Start server
echo "Starting handy-rules server..."
nohup "${SERVER_BIN}" --host "${HOST}" --port "${PORT}" --rules "${RULES}" \
    > "${SCRIPT_DIR}/server.log" 2>&1 &

# Poll for health
for i in $(seq 1 ${TIMEOUT}); do
    if curl -s --max-time 1 "${HEALTH_URL}" > /dev/null 2>&1; then
        echo "Server started successfully at http://${HOST}:${PORT}"
        exit 0
    fi
    sleep 1
done

echo "Failed to start server within ${TIMEOUT}s. Check server.log"
exit 1
```

### 12.2 Windows (start_local_llm.ps1)

```powershell
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ServerBin = Join-Path $ScriptDir "handy-rules.exe"
$Port = if ($env:PORT) { $env:PORT } else { "8080" }
$Host_ = if ($env:HOST) { $env:HOST } else { "127.0.0.1" }
$Rules = if ($env:RULES) { $env:RULES } else { Join-Path $ScriptDir "rules.json" }
$HealthUrl = "http://${Host_}:${Port}/health"
$Timeout = 5

# Check if running
try {
    $response = Invoke-WebRequest -Uri $HealthUrl -TimeoutSec 1 -UseBasicParsing
    Write-Host "Server already running at $HealthUrl"
    exit 0
} catch {}

# Start server
Write-Host "Starting handy-rules server..."
$LogFile = Join-Path $ScriptDir "server.log"
Start-Process -FilePath $ServerBin -ArgumentList "--host", $Host_, "--port", $Port, "--rules", $Rules `
    -RedirectStandardOutput $LogFile -RedirectStandardError $LogFile -WindowStyle Hidden

# Poll health
for ($i = 1; $i -le $Timeout; $i++) {
    Start-Sleep -Seconds 1
    try {
        $response = Invoke-WebRequest -Uri $HealthUrl -TimeoutSec 1 -UseBasicParsing
        Write-Host "Server started successfully at http://${Host_}:${Port}"
        exit 0
    } catch {}
}

Write-Host "Failed to start server within ${Timeout}s. Check server.log"
exit 1
```

---

## 13. systemd Service

### handy-rules.service

```ini
[Unit]
Description=Handy Local Rules Server
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/opt/handy-rules
ExecStart=/opt/handy-rules/handy-rules --port 8080 --rules /opt/handy-rules/rules.json
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Installation:
```bash
sudo cp handy-rules.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable handy-rules
sudo systemctl start handy-rules
```

---

## 14. Testing

### 14.1 Unit Tests

```rust
// tests/rule_tests.rs

#[test]
fn test_rule_priority_ordering() {
    let rules = vec![
        Rule { id: "low".into(), pattern: "a".into(), replacement: "1".into(), priority: 10, .. },
        Rule { id: "high".into(), pattern: "a".into(), replacement: "2".into(), priority: 100, .. },
    ];
    let engine = RuleEngine::new(rules);
    // High priority should apply first
}

#[test]
fn test_word_boundary_matching() {
    let rules = vec![
        Rule { id: "slash".into(), pattern: r"(?i)\bslash\b".into(), replacement: "/".into(), .. },
    ];
    let engine = RuleEngine::new(rules);
    assert_eq!(engine.apply("foo slash bar"), "foo / bar");
    assert_eq!(engine.apply("slashing"), "slashing"); // Should NOT match
}

#[test]
fn test_ignore_case() {
    let rules = vec![
        Rule { id: "dot".into(), pattern: r"(?i)\bdot\b".into(), replacement: ".".into(), .. },
    ];
    let engine = RuleEngine::new(rules);
    assert_eq!(engine.apply("DOT"), ".");
    assert_eq!(engine.apply("Dot"), ".");
    assert_eq!(engine.apply("dot"), ".");
}
```

### 14.2 Integration Tests

```rust
// tests/integration_tests.rs

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_app();
    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_chat_completion() {
    let app = create_app();
    let body = json!({
        "model": "local-rules",
        "messages": [{"role": "user", "content": "foo slash bar"}]
    });
    let response = app
        .oneshot(
            Request::post("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Parse response and verify content is "foo / bar"
}
```

### 14.3 Run Tests

```bash
cargo test
cargo test --all-features  # Include fuzzy tests
```

---

## 15. Handy Configuration

To configure Handy to use this local server:

1. Open Handy Settings → Post-Processing
2. Add new provider:
   - **ID:** `local`
   - **Base URL:** `http://127.0.0.1:8080`
   - **Models Endpoint:** (leave blank)
   - **API Key:** (if configured on server)
3. Enable Post-Processing
4. Select `local` as the active provider

---

## 16. Implementation Tasks

### Phase 1: Core Server (Priority: High)
- [ ] Initialize Cargo project with dependencies
- [ ] Implement data structures (Rule, Request, Response)
- [ ] Implement `/health` endpoint
- [ ] Implement `/v1/chat/completions` endpoint
- [ ] Implement basic rule engine with regex
- [ ] Implement rules.json loader
- [ ] Add CLI argument parsing

### Phase 2: Hot-Reload & Polish (Priority: Medium)
- [ ] Implement file watcher for hot-reload
- [ ] Add optional API key authentication
- [ ] Add structured logging with tracing
- [ ] Implement `/v1/models` endpoint
- [ ] Add CORS support

### Phase 3: Testing (Priority: High)
- [ ] Write unit tests for rule engine
- [ ] Write integration tests for API
- [ ] Create test rules.json with edge cases

### Phase 4: Distribution (Priority: Medium)
- [ ] Create launcher scripts (bash/PowerShell)
- [ ] Create systemd service file
- [ ] Set up cross-compilation
- [ ] Create release archives
- [ ] Write README with full documentation

### Phase 5: Optional Enhancements
- [ ] Implement fuzzy matching (feature flag)
- [ ] Add `/metrics` endpoint
- [ ] Create Windows service wrapper
- [ ] Create macOS launchd plist

---

## 17. Success Criteria

| Metric | Target |
|--------|--------|
| Response latency | ≤ 30ms for typical request |
| Startup time | ≤ 3s via launcher script |
| Test coverage | ≥ 80% for rule engine |
| Binary size | ≤ 5MB (release, stripped) |
| Memory usage | ≤ 20MB at idle |

---

## 18. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Handy sends unexpected request format | Flexible request parsing, fallback to raw body |
| Regex performance issues | Pre-compile regexes, lazy compilation |
| Hot-reload race conditions | Use RwLock for thread-safe rule updates |
| Cross-compilation failures | Test on CI with all target platforms |

---

## 19. Timeline Estimate

| Phase | Duration |
|-------|----------|
| Phase 1: Core Server | 1 day |
| Phase 2: Hot-Reload & Polish | 0.5 day |
| Phase 3: Testing | 0.5 day |
| Phase 4: Distribution | 1 day |
| **Total MVP** | **3 days** |
| Phase 5: Optional | +1-2 days |

---

## Appendix A: Example Request/Response Flow

**Input (from Handy):**
```json
{
  "model": "local-rules",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Please transcribe: foo slash bar dot com"}
  ]
}
```

**Processing:**
1. Extract user content: `"Please transcribe: foo slash bar dot com"`
2. Apply rule "slash" (priority 100): `"Please transcribe: foo / bar dot com"`
3. Apply rule "dot" (priority 80): `"Please transcribe: foo / bar . com"`

**Output:**
```json
{
  "id": "local-a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "object": "chat.completion",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Please transcribe: foo / bar . com"
    }
  }],
  "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
}
```
