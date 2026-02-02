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

use crate::config::{find_config_file, get_config_dir, Config};
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

        /// API key for authentication (overrides config file)
        #[arg(short, long, env = "API_KEY")]
        api_key: Option<String>,
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = load_config(&args.config);

    // Merge global CLI args
    let config = config.merge_with_args(
        None,
        None,
        args.rules,
        None,
        args.log_level,
    );

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .init();

    // Handle command
    match args.command {
        Some(Command::Serve { host, port, api_key }) => {
            let config = config.merge_with_args(host, port, None, api_key, None);
            run_server(config).await
        }
        Some(Command::Transform { text, stdin }) => {
            run_transform(&config, text, stdin)
        }
        Some(Command::Validate) => {
            run_validate(&config)
        }
        Some(Command::ListRules) => {
            run_list_rules(&config)
        }
        None => {
            // Default: start server (backward compatible)
            run_server(config).await
        }
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
        }
        None => {
            if config_path.is_some() {
                // User specified a config file but it wasn't found
                eprintln!(
                    "Config file not found: {}",
                    config_path.as_ref().unwrap()
                );
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
        }
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
        config.api_key,
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
        }
        Err(e) => {
            eprintln!("✗ Invalid rules: {}", e);
            std::process::exit(1);
        }
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
