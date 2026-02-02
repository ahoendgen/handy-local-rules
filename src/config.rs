//! Configuration management

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Default config directory name in home folder
const CONFIG_DIR_NAME: &str = ".handy-local-rules";

/// Default config file name
const CONFIG_FILE_NAME: &str = "config.json";

/// Default rules file name
const RULES_FILE_NAME: &str = "rules.json";

/// Get the default config directory (~/.handy-local-rules/)
pub fn get_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(CONFIG_DIR_NAME))
}

/// Get the default config file path (~/.handy-local-rules/config.json)
pub fn get_default_config_path() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join(CONFIG_FILE_NAME))
}

/// Get the default rules file path (~/.handy-local-rules/rules.json)
pub fn get_default_rules_path() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join(RULES_FILE_NAME))
}

/// Find config file in standard locations (in order of priority):
/// 1. Explicitly specified path (if provided)
/// 2. config.json in current directory
/// 3. ~/.handy-local-rules/config.json
pub fn find_config_file(explicit_path: Option<&Path>) -> Option<PathBuf> {
    // 1. Explicit path
    if let Some(path) = explicit_path {
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    // 2. Current directory
    let cwd_config = Path::new(CONFIG_FILE_NAME);
    if cwd_config.exists() {
        return Some(cwd_config.to_path_buf());
    }

    // 3. Home directory
    if let Some(home_config) = get_default_config_path() {
        if home_config.exists() {
            return Some(home_config);
        }
    }

    None
}

/// Find rules files in standard locations (in order of priority):
/// Returns paths that exist. Checks:
/// 1. Explicitly specified paths (from CLI)
/// 2. rules.json in current directory
/// 3. ~/.handy-local-rules/rules.json
/// 4. ~/.handy-local-rules/*.json (all JSON files in config dir)
pub fn find_rules_paths(explicit_paths: &[String]) -> Vec<String> {
    let mut paths = Vec::new();

    // 1. Add explicit paths first (they take priority)
    for path in explicit_paths {
        // Expand paths relative to home dir if they start with ~/
        let expanded = if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&path[2..]).to_string_lossy().to_string()
            } else {
                path.clone()
            }
        } else {
            path.clone()
        };
        paths.push(expanded);
    }

    // If no explicit paths, check default locations
    if explicit_paths.is_empty()
        || (explicit_paths.len() == 1 && explicit_paths[0] == RULES_FILE_NAME)
    {
        // 2. Current directory
        let cwd_rules = Path::new(RULES_FILE_NAME);
        if cwd_rules.exists() && !paths.contains(&RULES_FILE_NAME.to_string()) {
            paths.push(RULES_FILE_NAME.to_string());
        }

        // 3. Home directory rules file
        if let Some(home_rules) = get_default_rules_path() {
            if home_rules.exists() {
                let path_str = home_rules.to_string_lossy().to_string();
                if !paths.contains(&path_str) {
                    paths.push(path_str);
                }
            }
        }

        // 4. All JSON files in home config dir
        if let Some(config_dir) = get_config_dir() {
            if config_dir.exists() {
                let glob_pattern = config_dir.join("*.json").to_string_lossy().to_string();
                // Only add glob if there might be additional files
                if !paths.contains(&glob_pattern) {
                    paths.push(glob_pattern);
                }
            }
        }
    }

    // Remove duplicates while preserving order
    let mut unique_paths = Vec::new();
    for path in paths {
        if !unique_paths.contains(&path) {
            unique_paths.push(path);
        }
    }

    unique_paths
}

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Paths to rules files (files, directories, or glob patterns)
    /// Examples:
    /// - "rules.json" (single file)
    /// - ["rules.json", "custom-rules.json"] (multiple files)
    /// - "rules/*.json" (glob pattern)
    /// - "rules/" (directory - loads all .json files)
    #[serde(default = "default_rules_paths", alias = "rules_path")]
    pub rules_paths: RulesPaths,

    /// API key for authentication (optional)
    #[serde(default)]
    pub api_key: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Maximum transformation log entries to keep
    #[serde(default = "default_max_log_entries")]
    pub max_log_entries: usize,

    /// Enable CORS (cross-origin requests)
    #[serde(default = "default_cors_enabled")]
    pub cors_enabled: bool,

    /// Enable shell rules (security risk - disabled by default)
    /// Shell rules can execute arbitrary commands on your system.
    /// Only enable this if you trust all rule sources.
    #[serde(default)]
    pub enable_shell_rules: bool,
}

/// Rules paths can be a single string or an array of strings
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RulesPaths {
    Single(String),
    Multiple(Vec<String>),
}

impl RulesPaths {
    /// Convert to a Vec of paths
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            RulesPaths::Single(s) => vec![s.clone()],
            RulesPaths::Multiple(v) => v.clone(),
        }
    }
}

impl Default for RulesPaths {
    fn default() -> Self {
        RulesPaths::Single("rules.json".to_string())
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    61234
}

fn default_rules_paths() -> RulesPaths {
    RulesPaths::default()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_log_entries() -> usize {
    1000
}

fn default_cors_enabled() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            rules_paths: default_rules_paths(),
            api_key: None,
            log_level: default_log_level(),
            max_log_entries: default_max_log_entries(),
            cors_enabled: default_cors_enabled(),
            enable_shell_rules: false,
        }
    }
}

impl Config {
    /// Load configuration from a JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path.as_ref())?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from file if it exists, otherwise use defaults
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load(path) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    /// Get rules paths as Vec<String>
    pub fn get_rules_paths(&self) -> Vec<String> {
        self.rules_paths.to_vec()
    }

    /// Merge CLI arguments into config (CLI takes precedence)
    pub fn merge_with_args(
        mut self,
        host: Option<String>,
        port: Option<u16>,
        rules: Option<String>,
        api_key: Option<String>,
        log_level: Option<String>,
    ) -> Self {
        if let Some(h) = host {
            self.host = h;
        }
        if let Some(p) = port {
            self.port = p;
        }
        if let Some(r) = rules {
            // CLI rules path prepends to existing paths
            let mut paths = vec![r];
            paths.extend(self.rules_paths.to_vec());
            self.rules_paths = RulesPaths::Multiple(paths);
        }
        if api_key.is_some() {
            self.api_key = api_key;
        }
        if let Some(l) = log_level {
            self.log_level = l;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config_single_path() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"port": 9000, "rules_paths": "my-rules.json"}}"#).unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.get_rules_paths(), vec!["my-rules.json"]);
    }

    #[test]
    fn test_load_config_multiple_paths() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"rules_paths": ["rules.json", "custom/*.json", "extra/"]}}"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(
            config.get_rules_paths(),
            vec!["rules.json", "custom/*.json", "extra/"]
        );
    }

    #[test]
    fn test_load_config_with_alias() {
        let mut file = NamedTempFile::new().unwrap();
        // rules_path (singular) should also work
        writeln!(file, r#"{{"rules_path": "legacy.json"}}"#).unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.get_rules_paths(), vec!["legacy.json"]);
    }

    #[test]
    fn test_merge_with_args() {
        let config = Config::default();
        let merged = config.merge_with_args(
            Some("0.0.0.0".to_string()),
            Some(3000),
            Some("extra-rules.json".to_string()),
            None,
            None,
        );

        assert_eq!(merged.host, "0.0.0.0");
        assert_eq!(merged.port, 3000);
        // Extra rules prepended to default
        assert_eq!(
            merged.get_rules_paths(),
            vec!["extra-rules.json", "rules.json"]
        );
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.get_rules_paths(), vec!["rules.json"]);
    }
}
