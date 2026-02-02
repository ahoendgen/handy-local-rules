//! OpenAI-compatible request types

use serde::Deserialize;
use utoipa::ToSchema;

/// Chat completion request (OpenAI-compatible)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatCompletionRequest {
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
    /// 1. Last user message from messages array (strict: only "user" role)
    /// 2. prompt field
    /// 3. input field
    /// 4. text field
    ///
    /// Note: We intentionally do NOT fall back to the last message regardless of role.
    /// This prevents accidentally processing assistant prefills or system metadata.
    pub fn extract_user_content(&self) -> Option<String> {
        // Strategy 1: Get last user message (strict role check)
        if let Some(messages) = &self.messages {
            // Find the last user message - only accept role == "user"
            if let Some(msg) = messages.iter().rev().find(|m| m.role == "user") {
                return Some(msg.content.clone());
            }
            // No fallback to last message - this could be an assistant prefill
            // or system metadata which we should not process
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

    #[test]
    fn test_ignore_assistant_prefill() {
        // When an assistant message follows the user message (prefill),
        // we should still extract the user message, not the assistant prefill
        let request = ChatCompletionRequest {
            messages: Some(vec![
                Message {
                    role: "user".to_string(),
                    content: "Real Input".to_string(),
                },
                Message {
                    role: "assistant".to_string(),
                    content: "Prefill".to_string(),
                },
            ]),
            prompt: None,
            input: None,
            text: None,
        };

        // Should skip "Prefill" and find "Real Input"
        assert_eq!(
            request.extract_user_content(),
            Some("Real Input".to_string())
        );
    }
}
