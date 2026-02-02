//! HTTP request handlers

use crate::models::{
    ChatCompletionRequest, ChatCompletionResponse, HealthResponse, ModelsResponse, RuleInfo,
    RuleToggleResponse, RulesResponse, TransformationLogEntry, TransformationLogResponse,
};
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Json,
};

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let rules_count = state.rule_engine.rules_count();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        rules_loaded: rules_count,
    })
}

/// Chat completion endpoint (OpenAI-compatible)
///
/// Accepts text input and applies transformation rules.
/// Supports multiple input formats: messages array, prompt, input, or text fields.
#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, description = "Text transformed successfully", body = ChatCompletionResponse),
        (status = 400, description = "No user content found in request")
    ),
    tag = "Chat"
)]
pub async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    // Extract text to process
    let input_text = request.extract_user_content().ok_or_else(|| {
        tracing::warn!("No user content found in request");
        StatusCode::BAD_REQUEST
    })?;

    tracing::debug!("Processing input: {}", input_text);

    // Apply rules
    let processed_text = state.rule_engine.apply(&input_text);

    tracing::debug!("Output: {}", processed_text);

    // Build response
    let response = ChatCompletionResponse::new(processed_text);

    Ok(Json(response))
}

/// List available models endpoint
#[utoipa::path(
    get,
    path = "/v1/models",
    responses(
        (status = 200, description = "List of available models", body = ModelsResponse)
    ),
    tag = "Models"
)]
pub async fn list_models() -> Json<ModelsResponse> {
    Json(ModelsResponse::default())
}

/// Get transformation logs
///
/// Returns recent transformation logs showing which rules matched and how text was transformed.
#[utoipa::path(
    get,
    path = "/v1/logs",
    responses(
        (status = 200, description = "Transformation logs", body = TransformationLogResponse)
    ),
    tag = "Logs"
)]
pub async fn get_logs(State(state): State<AppState>) -> Json<TransformationLogResponse> {
    let logs = state.rule_engine.get_transformation_log();

    Json(TransformationLogResponse {
        logs: logs
            .into_iter()
            .map(|l| TransformationLogEntry {
                rule_id: l.rule_id,
                rule_type: l.rule_type,
                input: l.input,
                output: l.output,
                matched: l.matched,
            })
            .collect(),
    })
}

/// Clear transformation logs
#[utoipa::path(
    delete,
    path = "/v1/logs",
    responses(
        (status = 204, description = "Logs cleared successfully")
    ),
    tag = "Logs"
)]
pub async fn clear_logs(State(state): State<AppState>) -> StatusCode {
    state.rule_engine.clear_transformation_log();
    StatusCode::NO_CONTENT
}

/// Get all loaded rules
#[utoipa::path(
    get,
    path = "/v1/rules",
    responses(
        (status = 200, description = "List of all rules", body = RulesResponse)
    ),
    tag = "Rules"
)]
pub async fn get_rules(State(state): State<AppState>) -> Json<RulesResponse> {
    let rules = state.rule_engine.get_rules();

    Json(RulesResponse {
        count: rules.len(),
        rules: rules
            .into_iter()
            .map(|r| RuleInfo {
                id: r.id,
                description: r.description,
                rule_type: format!("{:?}", r.rule_type).to_lowercase(),
                pattern: r.pattern,
                replacement: r.replacement,
                priority: r.priority,
                enabled: r.enabled,
            })
            .collect(),
    })
}

/// Dashboard UI
pub async fn dashboard() -> Html<&'static str> {
    Html(include_str!("static/index.html"))
}

/// Toggle a rule's enabled state
///
/// Toggles the enabled/disabled state of a rule by its ID.
#[utoipa::path(
    post,
    path = "/v1/rules/{rule_id}/toggle",
    params(
        ("rule_id" = String, Path, description = "The rule ID to toggle")
    ),
    responses(
        (status = 200, description = "Rule toggled successfully", body = RuleToggleResponse),
        (status = 404, description = "Rule not found")
    ),
    tag = "Rules"
)]
pub async fn toggle_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<RuleToggleResponse>, StatusCode> {
    match state.rule_engine.toggle_rule(&rule_id) {
        Some(enabled) => {
            let status = if enabled { "enabled" } else { "disabled" };
            Ok(Json(RuleToggleResponse {
                id: rule_id.clone(),
                enabled,
                message: format!("Rule '{}' is now {}", rule_id, status),
            }))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}
