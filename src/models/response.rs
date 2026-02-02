//! OpenAI-compatible response types

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Namespace UUID for generating deterministic response IDs
/// This is a custom namespace for handy-local-rules
const RESPONSE_ID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x68, 0x61, 0x6e, 0x64, 0x79, 0x2d, 0x6c, 0x6f, // "handy-lo"
    0x63, 0x61, 0x6c, 0x2d, 0x72, 0x75, 0x6c, 0x65, // "cal-rule"
]);

/// Chat completion response (OpenAI-compatible)
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatCompletionResponse {
    /// Unique response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Response choices
    pub choices: Vec<Choice>,
    /// Token usage statistics
    pub usage: Usage,
}

impl ChatCompletionResponse {
    /// Create a new response with the given content
    /// Uses deterministic ID based on input content for caching/debugging
    pub fn new(input: &str, content: String) -> Self {
        // Generate deterministic UUID based on input content
        // This helps with client-side caching and debugging
        let id = Uuid::new_v5(&RESPONSE_ID_NAMESPACE, input.as_bytes());

        Self {
            id: format!("local-{}", id),
            object: "chat.completion".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ResponseMessage {
                    role: "assistant".to_string(),
                    content,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage::default(),
        }
    }
}

/// A single choice in the response
#[derive(Debug, Serialize, ToSchema)]
pub struct Choice {
    /// Choice index
    pub index: u32,
    /// Response message
    pub message: ResponseMessage,
    /// Reason for completion
    pub finish_reason: Option<String>,
}

/// Response message
#[derive(Debug, Serialize, ToSchema)]
pub struct ResponseMessage {
    /// Message role (always "assistant")
    pub role: String,
    /// Transformed content
    pub content: String,
}

/// Token usage (always zero for rule-based processing)
#[derive(Debug, Serialize, Default, ToSchema)]
pub struct Usage {
    /// Input tokens (always 0)
    pub prompt_tokens: u32,
    /// Output tokens (always 0)
    pub completion_tokens: u32,
    /// Total tokens (always 0)
    pub total_tokens: u32,
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Server status
    #[schema(example = "ok")]
    pub status: String,
    /// Server version
    #[schema(example = "0.1.0")]
    pub version: String,
    /// Number of loaded rules
    #[schema(example = 10)]
    pub rules_loaded: usize,
}

/// Models list response
#[derive(Debug, Serialize, ToSchema)]
pub struct ModelsResponse {
    /// Available models
    pub data: Vec<ModelInfo>,
}

/// Model info
#[derive(Debug, Serialize, ToSchema)]
pub struct ModelInfo {
    /// Model ID
    #[schema(example = "local-rules")]
    pub id: String,
    /// Object type
    #[schema(example = "model")]
    pub object: String,
}

impl Default for ModelsResponse {
    fn default() -> Self {
        Self {
            data: vec![ModelInfo {
                id: "local-rules".to_string(),
                object: "model".to_string(),
            }],
        }
    }
}

/// Transformation log response
#[derive(Debug, Serialize, ToSchema)]
pub struct TransformationLogResponse {
    /// Log entries
    pub logs: Vec<TransformationLogEntry>,
}

/// A single transformation log entry
#[derive(Debug, Serialize, ToSchema)]
pub struct TransformationLogEntry {
    /// Rule ID that was applied
    #[schema(example = "slash")]
    pub rule_id: String,
    /// Type of rule (Regex, Shell, Function)
    #[schema(example = "Regex")]
    pub rule_type: String,
    /// Input text before transformation
    #[schema(example = "foo slash bar")]
    pub input: String,
    /// Output text after transformation
    #[schema(example = "foo / bar")]
    pub output: String,
    /// Whether the rule matched and changed the text
    #[schema(example = true)]
    pub matched: bool,
}

/// Rules list response
#[derive(Debug, Serialize, ToSchema)]
pub struct RulesResponse {
    /// List of all loaded rules
    pub rules: Vec<RuleInfo>,
    /// Total count
    pub count: usize,
}

/// Rule info for API response
#[derive(Debug, Serialize, ToSchema)]
pub struct RuleInfo {
    /// Rule ID
    #[schema(example = "slash")]
    pub id: String,
    /// Description
    #[schema(example = "spoken 'slash' -> /")]
    pub description: Option<String>,
    /// Rule type (regex, shell, function)
    #[schema(example = "regex")]
    pub rule_type: String,
    /// Pattern or command
    #[schema(example = "(?i)\\bslash\\b")]
    pub pattern: String,
    /// Replacement (for regex rules)
    #[schema(example = "/")]
    pub replacement: String,
    /// Priority (higher = first)
    #[schema(example = 100)]
    pub priority: i32,
    /// Is rule enabled?
    #[schema(example = true)]
    pub enabled: bool,
}

/// Response for rule toggle/update operations
#[derive(Debug, Serialize, ToSchema)]
pub struct RuleToggleResponse {
    /// Rule ID
    #[schema(example = "slash")]
    pub id: String,
    /// New enabled state
    #[schema(example = true)]
    pub enabled: bool,
    /// Status message
    #[schema(example = "Rule 'slash' is now enabled")]
    pub message: String,
}
