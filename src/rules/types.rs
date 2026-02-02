//! Rule data structures

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Type of transformation to apply
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    /// Regex-based replacement (default)
    #[default]
    Regex,

    /// Execute a shell command (input via stdin, output via stdout)
    Shell,

    /// Built-in function (uppercase, lowercase, trim, etc.)
    Function,
}

/// A single transformation rule
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,

    /// Type of rule (regex, shell, function)
    #[serde(default, rename = "type")]
    pub rule_type: RuleType,

    /// For regex: pattern to match
    /// For shell: command to execute
    /// For function: function name
    pub pattern: String,

    /// For regex: replacement string (supports backreferences like $1, $2)
    /// For shell: not used (output is from stdout)
    /// For function: optional arguments
    #[serde(default)]
    pub replacement: String,

    /// Priority (higher = applied first)
    #[serde(default)]
    pub priority: i32,

    /// Whether the rule is active
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Case-insensitive matching (for regex rules)
    #[serde(default)]
    pub ignore_case: bool,

    /// Timeout in milliseconds for shell commands
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Reserved for fuzzy matching feature
    #[serde(default)]
    pub fuzzy_key: bool,

    /// Source file path (internal, not serialized to JSON output)
    #[serde(skip)]
    #[schema(hidden)]
    pub source_file: Option<String>,
}

fn default_enabled() -> bool {
    true
}

fn default_timeout() -> u64 {
    5000 // 5 seconds
}

impl Rule {
    /// Get the effective pattern, adding (?i) if ignore_case is set
    pub fn effective_pattern(&self) -> String {
        if self.ignore_case && !self.pattern.starts_with("(?i)") {
            format!("(?i){}", self.pattern)
        } else {
            self.pattern.clone()
        }
    }
}

/// Built-in transformation functions
#[derive(Debug, Clone, Copy)]
pub enum BuiltinFunction {
    /// Convert to uppercase
    Uppercase,
    /// Convert to lowercase
    Lowercase,
    /// Trim whitespace
    Trim,
    /// Trim start
    TrimStart,
    /// Trim end
    TrimEnd,
    /// Capitalize first letter
    Capitalize,
    /// Reverse string
    Reverse,
    /// Remove extra whitespace (multiple spaces -> single space)
    NormalizeWhitespace,
}

impl BuiltinFunction {
    /// Parse function name to enum
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "uppercase" | "upper" => Some(Self::Uppercase),
            "lowercase" | "lower" => Some(Self::Lowercase),
            "trim" => Some(Self::Trim),
            "trim_start" | "trimstart" | "ltrim" => Some(Self::TrimStart),
            "trim_end" | "trimend" | "rtrim" => Some(Self::TrimEnd),
            "capitalize" | "cap" => Some(Self::Capitalize),
            "reverse" => Some(Self::Reverse),
            "normalize_whitespace" | "normalize" => Some(Self::NormalizeWhitespace),
            _ => None,
        }
    }

    /// Apply the function to input text
    pub fn apply(&self, input: &str) -> String {
        match self {
            Self::Uppercase => input.to_uppercase(),
            Self::Lowercase => input.to_lowercase(),
            Self::Trim => input.trim().to_string(),
            Self::TrimStart => input.trim_start().to_string(),
            Self::TrimEnd => input.trim_end().to_string(),
            Self::Capitalize => {
                let mut chars = input.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            }
            Self::Reverse => input.chars().rev().collect(),
            Self::NormalizeWhitespace => {
                input.split_whitespace().collect::<Vec<_>>().join(" ")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_pattern_with_ignore_case() {
        let rule = Rule {
            id: "test".to_string(),
            description: None,
            rule_type: RuleType::Regex,
            pattern: r"\btest\b".to_string(),
            replacement: "TEST".to_string(),
            priority: 0,
            enabled: true,
            ignore_case: true,
            timeout_ms: 5000,
            fuzzy_key: false,
            source_file: None,
        };

        assert_eq!(rule.effective_pattern(), r"(?i)\btest\b");
    }

    #[test]
    fn test_builtin_functions() {
        assert_eq!(BuiltinFunction::Uppercase.apply("hello"), "HELLO");
        assert_eq!(BuiltinFunction::Lowercase.apply("HELLO"), "hello");
        assert_eq!(BuiltinFunction::Trim.apply("  hello  "), "hello");
        assert_eq!(BuiltinFunction::Capitalize.apply("hello"), "Hello");
        assert_eq!(BuiltinFunction::Reverse.apply("hello"), "olleh");
        assert_eq!(
            BuiltinFunction::NormalizeWhitespace.apply("hello   world  "),
            "hello world"
        );
    }

    #[test]
    fn test_function_from_name() {
        assert!(BuiltinFunction::from_name("uppercase").is_some());
        assert!(BuiltinFunction::from_name("UPPER").is_some());
        assert!(BuiltinFunction::from_name("unknown").is_none());
    }
}
