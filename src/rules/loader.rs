//! Rules file loading and hot-reload

use super::engine::RuleEngine;
use super::types::Rule;
use crate::error::AppError;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Load rules from a JSON file
pub fn load_rules(path: &PathBuf) -> Result<Vec<Rule>, AppError> {
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::RulesLoadError(format!("Failed to read {}: {}", path.display(), e))
    })?;

    let mut rules: Vec<Rule> = serde_json::from_str(&content).map_err(|e| {
        AppError::RulesLoadError(format!("Failed to parse {}: {}", path.display(), e))
    })?;

    // Set source file path for each rule
    let source_path = path.to_string_lossy().to_string();
    for rule in &mut rules {
        rule.source_file = Some(source_path.clone());
    }

    Ok(rules)
}

/// Save rules to their source file
/// Only saves rules that belong to the specified file
pub fn save_rules_to_file(path: &str, rules: &[Rule]) -> Result<(), AppError> {
    // Filter rules that belong to this file
    let rules_for_file: Vec<&Rule> = rules
        .iter()
        .filter(|r| r.source_file.as_deref() == Some(path))
        .collect();

    if rules_for_file.is_empty() {
        return Err(AppError::RulesLoadError(format!(
            "No rules found for file: {}",
            path
        )));
    }

    // Read the original file to preserve formatting as much as possible
    let content = fs::read_to_string(path).map_err(|e| {
        AppError::RulesLoadError(format!("Failed to read {}: {}", path, e))
    })?;

    // Parse the original JSON to get the structure
    let mut original: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| {
        AppError::RulesLoadError(format!("Failed to parse {}: {}", path, e))
    })?;

    // Update the enabled field for each rule
    for rule in rules_for_file {
        if let Some(json_rule) = original.iter_mut().find(|r| {
            r.get("id").and_then(|v| v.as_str()) == Some(&rule.id)
        }) {
            if let Some(obj) = json_rule.as_object_mut() {
                obj.insert("enabled".to_string(), serde_json::Value::Bool(rule.enabled));
            }
        }
    }

    // Write back with pretty formatting
    let output = serde_json::to_string_pretty(&original).map_err(|e| {
        AppError::RulesLoadError(format!("Failed to serialize rules: {}", e))
    })?;

    fs::write(path, output + "\n").map_err(|e| {
        AppError::RulesLoadError(format!("Failed to write {}: {}", path, e))
    })?;

    tracing::info!("Saved rules to {}", path);

    Ok(())
}

/// Load rules from multiple sources (files, directories, or glob patterns)
pub fn load_rules_from_paths(paths: &[String]) -> Result<Vec<Rule>, AppError> {
    let mut all_rules = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);

        if path.is_dir() {
            // Load all .json files from directory
            let entries = fs::read_dir(path).map_err(|e| {
                AppError::RulesLoadError(format!("Failed to read directory {}: {}", path_str, e))
            })?;

            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "json").unwrap_or(false) {
                    tracing::debug!("Loading rules from {:?}", file_path);
                    let rules = load_rules(&file_path)?;
                    all_rules.extend(rules);
                }
            }
        } else if path.exists() {
            // Load single file
            tracing::debug!("Loading rules from {:?}", path);
            let rules = load_rules(&path.to_path_buf())?;
            all_rules.extend(rules);
        } else {
            // Try as glob pattern
            if let Ok(entries) = glob::glob(path_str) {
                for entry in entries.flatten() {
                    tracing::debug!("Loading rules from {:?}", entry);
                    let rules = load_rules(&entry)?;
                    all_rules.extend(rules);
                }
            } else {
                return Err(AppError::RulesLoadError(format!(
                    "Path not found: {}",
                    path_str
                )));
            }
        }
    }

    Ok(all_rules)
}

/// Watch the rules file for changes and reload when modified
pub fn watch_rules_file(path: PathBuf, engine: Arc<RuleEngine>) -> Result<(), AppError> {
    let path_clone = path.clone();

    // Create watcher
    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_)
                    ) {
                        tracing::info!("Rules file changed, reloading...");
                        if let Err(e) = engine.reload() {
                            tracing::error!("Failed to reload rules: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("File watcher error: {}", e);
                }
            }
        })?;

    // Watch the rules file
    watcher.watch(&path_clone, RecursiveMode::NonRecursive)?;

    // Keep watcher alive by leaking it (it needs to live for the duration of the program)
    // In a real application, you might want to store this in the AppState
    std::mem::forget(watcher);

    tracing::info!("Watching {:?} for changes", path_clone);

    Ok(())
}
