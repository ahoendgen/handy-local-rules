//! OpenAI-compatible request types

use serde::Deserialize;
use utoipa::ToSchema;

/// Chat completion request (OpenAI-compatible)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatCompletionRequest {
    /// Model name (ignored, always uses local rules)
    #[serde(default)]
    #[schema(example = "local-rules")]
    pub model: Option<String>,

    /// Chat messages
    #[serde(default)]
    pub messages: Option<Vec<Message>>,

    /// Alternative: raw prompt string
    #[serde(default)]
    #[schema(example = "Hello slash world")]
    pub prompt: Option<String>,

    /// Alternative: input field
    #[serde(default)]
    pub input: Option<String>,

    /// Alternative: text field
    #[serde(default)]
    pub text: Option<String>,
}

/// A single message in the chat
#[derive(Debug, Deserialize, ToSchema)]
pub struct Message {
    /// Message role (system, user, assistant)
    #[schema(example = "user")]
    pub role: String,
    /// Message content
    #[schema(example = "Please transform: foo slash bar dot com")]
    pub content: String,
}

impl ChatCompletionRequest {
    /// Extract the user content to process.
    ///
    /// Tries multiple strategies:
    /// 1. Last user message from messages array
    /// 2. prompt field
    /// 3. input field
    /// 4. text field
    pub fn extract_user_content(&self) -> Option<String> {
        // Strategy 1: Get last user message
        if let Some(messages) = &self.messages {
            // Find the last user message
            if let Some(msg) = messages.iter().rev().find(|m| m.role == "user") {
                return Some(msg.content.clone());
            }
            // Fallback: use last message regardless of role
            if let Some(msg) = messages.last() {
                return Some(msg.content.clone());
            }
        }

        // Strategy 2: prompt field
        if let Some(prompt) = &self.prompt {
            if !prompt.is_empty() {
                return Some(prompt.clone());
            }
        }

        // Strategy 3: input field
        if let Some(input) = &self.input {
            if !input.is_empty() {
                return Some(input.clone());
            }
        }

        // Strategy 4: text field
        if let Some(text) = &self.text {
            if !text.is_empty() {
                return Some(text.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_messages() {
        let request = ChatCompletionRequest {
            model: None,
            messages: Some(vec![
                Message {
                    role: "system".to_string(),
                    content: "You are helpful.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Hello world".to_string(),
                },
            ]),
            prompt: None,
            input: None,
            text: None,
        };

        assert_eq!(
            request.extract_user_content(),
            Some("Hello world".to_string())
        );
    }

    #[test]
    fn test_extract_from_prompt() {
        let request = ChatCompletionRequest {
            model: None,
            messages: None,
            prompt: Some("Test prompt".to_string()),
            input: None,
            text: None,
        };

        assert_eq!(
            request.extract_user_content(),
            Some("Test prompt".to_string())
        );
    }
}
