//! Rule application engine

use super::loader;
use super::types::{BuiltinFunction, Rule, RuleType};
use crate::error::AppError;
use notify::RecommendedWatcher;
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// Record of a single transformation
#[derive(Debug, Clone)]
pub struct TransformationLog {
    pub rule_id: String,
    pub rule_type: String,
    pub input: String,
    pub output: String,
    pub matched: bool,
}

/// The rule engine that applies transformation rules to text
pub struct RuleEngine {
    /// Paths to rules files
    rules_paths: Vec<String>,

    /// Currently loaded rules
    rules: RwLock<Vec<Rule>>,

    /// Compiled regex cache
    regex_cache: RwLock<HashMap<String, Regex>>,

    /// Transformation log (most recent transformations)
    transformation_log: RwLock<Vec<TransformationLog>>,

    /// Maximum log entries to keep
    max_log_entries: usize,

    /// Whether shell rules are enabled (security feature)
    enable_shell_rules: bool,

    /// File watchers (kept alive for the lifetime of the engine)
    #[allow(dead_code)]
    watchers: Mutex<Vec<RecommendedWatcher>>,
}

impl RuleEngine {
    /// Create a new rule engine and load rules from the given path
    pub fn new(rules_path: &str, enable_shell_rules: bool) -> Result<Self, AppError> {
        Self::new_from_paths(&[rules_path.to_string()], enable_shell_rules)
    }

    /// Create a new rule engine and load rules from multiple paths
    pub fn new_from_paths(paths: &[String], enable_shell_rules: bool) -> Result<Self, AppError> {
        let rules = loader::load_rules_from_paths(paths)?;

        // Count and warn about shell rules
        let shell_rule_count = rules
            .iter()
            .filter(|r| matches!(r.rule_type, RuleType::Shell))
            .count();
        if shell_rule_count > 0 {
            if enable_shell_rules {
                tracing::warn!(
                    "⚠️  {} shell rule(s) loaded. Shell rules can execute arbitrary commands!",
                    shell_rule_count
                );
            } else {
                tracing::warn!(
                    "⚠️  {} shell rule(s) found but DISABLED. Set enable_shell_rules=true to enable.",
                    shell_rule_count
                );
            }
        }

        tracing::info!("Loaded {} rules from {:?}", rules.len(), paths);

        let engine = Self {
            rules_paths: paths.to_vec(),
            rules: RwLock::new(rules),
            regex_cache: RwLock::new(HashMap::new()),
            transformation_log: RwLock::new(Vec::new()),
            max_log_entries: 1000,
            enable_shell_rules,
            watchers: Mutex::new(Vec::new()),
        };

        // Pre-compile all regexes
        engine.compile_regexes()?;

        Ok(engine)
    }

    /// Get the number of loaded rules
    pub fn rules_count(&self) -> usize {
        self.rules.read().unwrap().len()
    }

    /// Get all loaded rules
    pub fn get_rules(&self) -> Vec<Rule> {
        self.rules.read().unwrap().clone()
    }

    /// Toggle a rule's enabled state and persist to file
    /// Returns the new enabled state, or None if rule not found
    pub fn toggle_rule(&self, rule_id: &str) -> Option<bool> {
        let (new_state, source_file) = {
            let mut rules = self.rules.write().unwrap();

            let mut result = None;
            for rule in rules.iter_mut() {
                if rule.id == rule_id {
                    rule.enabled = !rule.enabled;
                    result = Some((rule.enabled, rule.source_file.clone()));
                    break;
                }
            }
            match result {
                Some((state, source)) => (state, source),
                None => {
                    tracing::warn!("Rule '{}' not found", rule_id);
                    return None;
                },
            }
        };

        tracing::info!(
            "Rule '{}' is now {}",
            rule_id,
            if new_state { "enabled" } else { "disabled" }
        );

        // Persist change to file
        if let Some(ref path) = source_file {
            let rules = self.rules.read().unwrap();
            if let Err(e) = loader::save_rules_to_file(path, &rules) {
                tracing::error!("Failed to persist rule change: {}", e);
            }
        } else {
            tracing::warn!("Rule '{}' has no source file, cannot persist", rule_id);
        }

        Some(new_state)
    }

    /// Set a rule's enabled state explicitly
    /// Returns the new enabled state, or None if rule not found
    pub fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> Option<bool> {
        let mut rules = self.rules.write().unwrap();

        for rule in rules.iter_mut() {
            if rule.id == rule_id {
                rule.enabled = enabled;
                tracing::info!(
                    "Rule '{}' is now {}",
                    rule_id,
                    if enabled { "enabled" } else { "disabled" }
                );
                return Some(enabled);
            }
        }

        tracing::warn!("Rule '{}' not found", rule_id);
        None
    }

    /// Get recent transformation logs
    pub fn get_transformation_log(&self) -> Vec<TransformationLog> {
        self.transformation_log.read().unwrap().clone()
    }

    /// Clear transformation log
    pub fn clear_transformation_log(&self) {
        self.transformation_log.write().unwrap().clear();
    }

    /// Apply all enabled rules to the input text
    /// Rules are pre-sorted by priority during load, so this is O(N) not O(N log N)
    pub fn apply(&self, text: &str) -> String {
        let rules = self.rules.read().unwrap();
        let cache = self.regex_cache.read().unwrap();

        let mut result = text.to_string();

        // Rules are pre-sorted by priority (descending) during load
        for rule in rules.iter().filter(|r| r.enabled) {
            // Skip shell rules if not enabled (security)
            if matches!(rule.rule_type, RuleType::Shell) && !self.enable_shell_rules {
                tracing::trace!("Skipping shell rule '{}' (shell rules disabled)", rule.id);
                continue;
            }

            let before = result.clone();

            result = match rule.rule_type {
                RuleType::Regex => self.apply_regex_rule(rule, &result, &cache),
                RuleType::Shell => self.apply_shell_rule(rule, &result),
                RuleType::Function => self.apply_function_rule(rule, &result),
            };

            // Log transformation
            let matched = before != result;
            self.log_transformation(TransformationLog {
                rule_id: rule.id.clone(),
                rule_type: format!("{:?}", rule.rule_type),
                input: before.clone(),
                output: result.clone(),
                matched,
            });

            if matched {
                tracing::debug!(
                    "Rule '{}' ({:?}) transformed: '{}' -> '{}'",
                    rule.id,
                    rule.rule_type,
                    before,
                    result
                );

                // Stop processing if rule has stop_on_match flag
                if rule.stop_on_match {
                    tracing::debug!(
                        "Rule '{}' has stop_on_match=true, stopping processing",
                        rule.id
                    );
                    break;
                }
            }
        }

        result
    }

    /// Apply a regex-based rule
    fn apply_regex_rule(&self, rule: &Rule, text: &str, cache: &HashMap<String, Regex>) -> String {
        if let Some(regex) = cache.get(&rule.id) {
            regex.replace_all(text, &rule.replacement).to_string()
        } else {
            text.to_string()
        }
    }

    /// Apply a shell command rule
    fn apply_shell_rule(&self, rule: &Rule, text: &str) -> String {
        let timeout = Duration::from_millis(rule.timeout_ms);

        match self.execute_shell_command(&rule.pattern, text, timeout) {
            Ok(output) => output,
            Err(e) => {
                tracing::error!("Shell rule '{}' failed: {}", rule.id, e);
                text.to_string() // Return original on error
            },
        }
    }

    /// Execute a shell command with input via stdin
    fn execute_shell_command(
        &self,
        command: &str,
        input: &str,
        timeout: Duration,
    ) -> Result<String, AppError> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::RulesLoadError(format!("Failed to spawn shell: {}", e)))?;

        // Write input to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .map_err(|e| AppError::RulesLoadError(format!("Failed to write stdin: {}", e)))?;
        }

        // Wait for output with timeout
        let output = child
            .wait_with_output()
            .map_err(|e| AppError::RulesLoadError(format!("Command failed: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::RulesLoadError(format!(
                "Command exited with {}: {}",
                output.status, stderr
            )))
        }
    }

    /// Apply a built-in function rule
    fn apply_function_rule(&self, rule: &Rule, text: &str) -> String {
        match BuiltinFunction::from_name(&rule.pattern) {
            Some(func) => func.apply(text),
            None => {
                tracing::warn!("Unknown function '{}' in rule '{}'", rule.pattern, rule.id);
                text.to_string()
            },
        }
    }

    /// Log a transformation
    fn log_transformation(&self, log: TransformationLog) {
        let mut logs = self.transformation_log.write().unwrap();

        logs.push(log);

        // Trim if too many entries
        if logs.len() > self.max_log_entries {
            let drain_count = logs.len() - self.max_log_entries;
            logs.drain(0..drain_count);
        }
    }

    /// Reload rules from all paths
    pub fn reload(&self) -> Result<(), AppError> {
        let new_rules = loader::load_rules_from_paths(&self.rules_paths)?;

        tracing::info!(
            "Reloading {} rules from {:?}",
            new_rules.len(),
            self.rules_paths
        );

        // Update rules
        {
            let mut rules = self.rules.write().unwrap();
            *rules = new_rules;
        }

        // Recompile regexes
        self.compile_regexes()?;

        Ok(())
    }

    /// Start watching rules files for changes
    /// Watchers are stored in the engine to keep them alive
    pub fn watch_for_changes(self: Arc<Self>) -> Result<(), AppError> {
        let mut watchers_guard = self.watchers.lock().unwrap();

        for path in &self.rules_paths {
            let watcher = loader::watch_rules_file(PathBuf::from(path), self.clone())?;
            watchers_guard.push(watcher);
        }

        Ok(())
    }

    /// Compile all regex patterns and cache them
    fn compile_regexes(&self) -> Result<(), AppError> {
        let rules = self.rules.read().unwrap();
        let mut cache = self.regex_cache.write().unwrap();

        cache.clear();

        for rule in rules.iter() {
            if matches!(rule.rule_type, RuleType::Regex) {
                let pattern = rule.effective_pattern();
                match Regex::new(&pattern) {
                    Ok(regex) => {
                        cache.insert(rule.id.clone(), regex);
                    },
                    Err(e) => {
                        tracing::error!("Invalid regex in rule '{}': {}", rule.id, e);
                        return Err(AppError::InvalidRegex(e));
                    },
                }
            }
        }

        tracing::debug!("Compiled {} regex patterns", cache.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_rules_file(rules: &[Rule]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let json = serde_json::to_string(rules).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_apply_regex_rule() {
        let rules = vec![Rule {
            id: "slash".to_string(),
            description: Some("slash -> /".to_string()),
            rule_type: RuleType::Regex,
            pattern: r"(?i)\bslash\b".to_string(),
            replacement: "/".to_string(),
            priority: 100,
            enabled: true,
            ignore_case: false,
            timeout_ms: 5000,
            stop_on_match: false,
        }];

        let file = create_test_rules_file(&rules);
        let engine = RuleEngine::new(file.path().to_str().unwrap(), false).unwrap();

        assert_eq!(engine.apply("foo slash bar"), "foo / bar");
    }

    #[test]
    fn test_apply_function_rule() {
        let rules = vec![Rule {
            id: "upper".to_string(),
            description: Some("Convert to uppercase".to_string()),
            rule_type: RuleType::Function,
            pattern: "uppercase".to_string(),
            replacement: String::new(),
            priority: 100,
            enabled: true,
            ignore_case: false,
            timeout_ms: 5000,
            stop_on_match: false,
        }];

        let file = create_test_rules_file(&rules);
        let engine = RuleEngine::new(file.path().to_str().unwrap(), false).unwrap();

        assert_eq!(engine.apply("hello world"), "HELLO WORLD");
    }

    #[test]
    fn test_apply_shell_rule() {
        let rules = vec![Rule {
            id: "echo".to_string(),
            description: Some("Echo with prefix".to_string()),
            rule_type: RuleType::Shell,
            pattern: "cat | tr 'a-z' 'A-Z'".to_string(),
            replacement: String::new(),
            priority: 100,
            enabled: true,
            ignore_case: false,
            timeout_ms: 5000,
            stop_on_match: false,
        }];

        let file = create_test_rules_file(&rules);
        // Shell rules need enable_shell_rules=true
        let engine = RuleEngine::new(file.path().to_str().unwrap(), true).unwrap();

        assert_eq!(engine.apply("hello"), "HELLO");
    }

    #[test]
    fn test_transformation_log() {
        let rules = vec![Rule {
            id: "test".to_string(),
            description: None,
            rule_type: RuleType::Regex,
            pattern: r"foo".to_string(),
            replacement: "bar".to_string(),
            priority: 100,
            enabled: true,
            ignore_case: false,
            timeout_ms: 5000,
            stop_on_match: false,
        }];

        let file = create_test_rules_file(&rules);
        let engine = RuleEngine::new(file.path().to_str().unwrap(), false).unwrap();

        engine.apply("foo test");

        let logs = engine.get_transformation_log();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].matched);
        assert_eq!(logs[0].rule_id, "test");
    }
}
