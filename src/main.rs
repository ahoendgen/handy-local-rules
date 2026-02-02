//! handy-local-rules - Local rule-based text transformer
//!
//! A lightweight HTTP server that provides an OpenAI-compatible API
//! for transforming text using regex-based rules.
//!
//! Can be used as:
//! - HTTP server: `handy-rules serve`
//! - CLI tool: `handy-rules transform "text to transform"`

mod config;
mod error;
mod handlers;
mod models;
mod rules;
mod server;

use crate::config::{Config, find_config_file, get_config_dir};
use crate::rules::RuleEngine;
use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "handy-rules")]
#[command(about = "Local rule-based text transformer with OpenAI-compatible API")]
#[command(version)]
struct Args {
    /// Path to configuration file (JSON).
    /// Searches: ./config.json, ~/.handy-local-rules/config.json
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Path to rules file/directory/glob (overrides config file).
    /// Default: ./rules.json, ~/.handy-local-rules/*.json
    #[arg(short, long, global = true)]
    rules: Option<String>,

    /// Log level: trace, debug, info, warn, error (overrides config file)
    #[arg(short, long, global = true)]
    log_level: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the HTTP server
    Serve {
        /// Host to bind to (overrides config file)
        #[arg(short = 'H', long)]
        host: Option<String>,

        /// Port to listen on (overrides config file)
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Transform text using rules (CLI mode)
    Transform {
        /// Text to transform (if not provided, reads from stdin)
        text: Option<String>,

        /// Read input line by line from stdin
        #[arg(short, long)]
        stdin: bool,
    },

    /// Validate rules file
    Validate,

    /// List all loaded rules
    #[command(name = "list-rules")]
    ListRules,

    /// Show service installation status (macOS)
    Status,

    /// Setup: copy rules to user config directory (~/.handy-local-rules/)
    Setup {
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
    },

    /// Open the web dashboard in the browser
    Dashboard,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = load_config(&args.config);

    // Merge global CLI args
    let config = config.merge_with_args(None, None, args.rules, args.log_level);

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .init();

    // Handle command
    match args.command {
        Some(Command::Serve { host, port }) => {
            let config = config.merge_with_args(host, port, None, None);
            run_server(config).await
        },
        Some(Command::Transform { text, stdin }) => run_transform(&config, text, stdin),
        Some(Command::Validate) => run_validate(&config),
        Some(Command::ListRules) => run_list_rules(&config),
        Some(Command::Status) => run_status(&config).await,
        Some(Command::Setup { force }) => run_setup(force),
        Some(Command::Dashboard) => run_dashboard(&config),
        None => {
            // Default: start server (backward compatible)
            run_server(config).await
        },
    }
}

fn load_config(config_path: &Option<String>) -> Config {
    // Find config file in standard locations
    let explicit_path = config_path.as_ref().map(std::path::Path::new);
    let found_config = find_config_file(explicit_path);

    match found_config {
        Some(path) => {
            eprintln!("Loading config from: {}", path.display());
            Config::load(&path).unwrap_or_else(|e| {
                eprintln!("Error loading config from {}: {}", path.display(), e);
                std::process::exit(1);
            })
        },
        None => {
            if config_path.is_some() {
                // User specified a config file but it wasn't found
                eprintln!("Config file not found: {}", config_path.as_ref().unwrap());
                std::process::exit(1);
            }
            // No config file found, use defaults
            if let Some(config_dir) = get_config_dir() {
                eprintln!(
                    "No config file found. Using defaults. (Hint: create config at {})",
                    config_dir.join("config.json").display()
                );
            }
            Config::default()
        },
    }
}

async fn run_server(config: Config) -> anyhow::Result<()> {
    tracing::info!(
        "Starting handy-rules server on {}:{}",
        config.host,
        config.port
    );
    tracing::debug!("Configuration: {:?}", config);

    server::run(
        &config.host,
        config.port,
        &config.get_rules_paths(),
        config.enable_shell_rules,
    )
    .await
}

fn run_transform(config: &Config, text: Option<String>, stdin: bool) -> anyhow::Result<()> {
    let engine = RuleEngine::new_from_paths(&config.get_rules_paths(), config.enable_shell_rules)?;

    if let Some(input) = text {
        // Transform provided text
        let output = engine.apply(&input);
        println!("{}", output);
    } else if stdin {
        // Read and transform line by line
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for line in stdin.lock().lines() {
            let line = line?;
            let output = engine.apply(&line);
            writeln!(stdout, "{}", output)?;
        }
    } else {
        // Read all from stdin, transform, output
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let output = engine.apply(input.trim());
        println!("{}", output);
    }

    Ok(())
}

fn run_validate(config: &Config) -> anyhow::Result<()> {
    let paths = config.get_rules_paths();
    match RuleEngine::new_from_paths(&paths, config.enable_shell_rules) {
        Ok(engine) => {
            println!("✓ Rules files are valid");
            println!("  Loaded {} rules from {:?}", engine.rules_count(), paths);
            Ok(())
        },
        Err(e) => {
            eprintln!("✗ Invalid rules: {}", e);
            std::process::exit(1);
        },
    }
}

fn run_list_rules(config: &Config) -> anyhow::Result<()> {
    let paths = config.get_rules_paths();
    let engine = RuleEngine::new_from_paths(&paths, config.enable_shell_rules)?;
    let rules = engine.get_rules();

    println!("Loaded {} rules from {:?}:\n", rules.len(), paths);

    for rule in rules {
        let status = if rule.enabled { "✓" } else { "✗" };
        let rule_type = format!("{:?}", rule.rule_type).to_lowercase();
        println!(
            "{} [{}] {} (priority: {}, type: {})",
            status,
            rule.id,
            rule.description.unwrap_or_default(),
            rule.priority,
            rule_type
        );
        println!("    pattern: {}", rule.pattern);
        if !rule.replacement.is_empty() {
            println!("    replacement: {}", rule.replacement);
        }
        println!();
    }

    Ok(())
}

fn run_dashboard(config: &Config) -> anyhow::Result<()> {
    let url = format!("http://{}:{}", config.host, config.port);

    println!("Opening dashboard: {}", url);

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()?;
    }

    Ok(())
}

fn run_setup(force: bool) -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;

    let config_dir =
        get_config_dir().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    println!("Setting up handy-local-rules...\n");
    println!("Target directory: {}\n", config_dir.display());

    // Create config directory
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        println!("✓ Created config directory");
    } else {
        println!("• Config directory already exists");
    }

    // Find source rules directory (relative to executable or current dir)
    let source_rules = Path::new("rules");
    if !source_rules.exists() {
        return Err(anyhow::anyhow!(
            "Rules directory not found. Run this command from the project root."
        ));
    }

    // Copy rules directory
    let dest_rules = config_dir.join("rules");
    if !dest_rules.exists() {
        fs::create_dir_all(&dest_rules)?;
    }

    let mut copied = 0;
    let mut skipped = 0;

    // Walk through rules directory and copy .json files
    for entry in glob::glob("rules/**/*.json")? {
        let entry = entry?;
        let relative = entry.strip_prefix("rules").unwrap();
        let dest_file = dest_rules.join(relative);

        // Create parent directories
        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent)?;
        }

        if dest_file.exists() && !force {
            println!("  • Skipped: rules/{} (exists)", relative.display());
            skipped += 1;
        } else {
            fs::copy(&entry, &dest_file)?;
            println!("  ✓ Copied:  rules/{}", relative.display());
            copied += 1;
        }
    }

    println!("\nRules: {} copied, {} skipped", copied, skipped);

    // Copy config.example.json as config.json
    let config_dest = config_dir.join("config.json");
    let config_src = Path::new("config.example.json");

    if config_src.exists() {
        if config_dest.exists() && !force {
            println!("• Config file already exists (use --force to overwrite)");
        } else {
            fs::copy(config_src, &config_dest)?;
            println!("✓ Copied config.example.json -> config.json");
        }
    }

    println!("\nSetup complete!");
    println!("\nNext steps:");
    println!("  1. Edit {}/config.json", config_dir.display());
    println!("  2. Run: handy-rules serve");
    println!("  3. Or install as service: make install");

    Ok(())
}

async fn run_status(config: &Config) -> anyhow::Result<()> {
    use std::path::Path;
    use std::process::Command as ProcessCommand;

    const SERVICE_NAME: &str = "dev.a9g.handy-local-rules";

    println!("handy-local-rules Status\n");

    // Check if plist is installed
    let plist_path = dirs::home_dir().map(|h| {
        h.join("Library/LaunchAgents")
            .join(format!("{}.plist", SERVICE_NAME))
    });

    let plist_installed = plist_path.as_ref().is_some_and(|p| p.exists());
    println!(
        "Service installed: {}",
        if plist_installed { "✓ yes" } else { "✗ no" }
    );

    // Check if service is running (launchctl list)
    let service_running = ProcessCommand::new("launchctl")
        .args(["list", SERVICE_NAME])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    println!(
        "Service running:   {}",
        if service_running { "✓ yes" } else { "✗ no" }
    );

    // Check if CLI symlink exists
    let cli_installed = Path::new("/usr/local/bin/handy-rules").exists();
    println!(
        "CLI installed:     {}",
        if cli_installed { "✓ yes" } else { "✗ no" }
    );

    // Check health endpoint
    let health_url = format!("http://{}:{}/health", config.host, config.port);
    let health_ok = reqwest::get(&health_url)
        .await
        .is_ok_and(|r| r.status().is_success());

    println!(
        "Health check:      {}",
        if health_ok {
            format!("✓ ok ({})", health_url)
        } else {
            format!("✗ unavailable ({})", health_url)
        }
    );

    // Show config location
    println!("\nConfiguration:");
    if let Some(config_dir) = get_config_dir() {
        println!("  Config dir: {}", config_dir.display());
    }
    println!("  Server:     {}:{}", config.host, config.port);

    Ok(())
}
