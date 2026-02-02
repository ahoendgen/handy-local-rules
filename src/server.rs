//! HTTP server setup and routing

use crate::handlers;
use crate::models::{
    ChatCompletionRequest, ChatCompletionResponse, Choice, HealthResponse, Message, ModelInfo,
    ModelsResponse, ResponseMessage, RuleInfo, RuleToggleResponse, RulesResponse,
    TransformationLogEntry, TransformationLogResponse, Usage,
};
use crate::rules::RuleEngine;
use axum::{routing::delete, routing::get, routing::post, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub rule_engine: Arc<RuleEngine>,
    pub api_key: Option<String>,
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Handy Local Rules API",
        version = "0.1.0",
        description = "Local rule-based text transformer with OpenAI-compatible API.\n\nTransform transcribed text using regex rules, built-in functions, or shell commands.",
        license(name = "MIT")
    ),
    paths(
        handlers::health,
        handlers::chat_completions,
        handlers::list_models,
        handlers::get_logs,
        handlers::clear_logs,
        handlers::get_rules,
        handlers::toggle_rule,
    ),
    components(schemas(
        ChatCompletionRequest,
        ChatCompletionResponse,
        Message,
        Choice,
        ResponseMessage,
        Usage,
        HealthResponse,
        ModelsResponse,
        ModelInfo,
        TransformationLogResponse,
        TransformationLogEntry,
        RulesResponse,
        RuleInfo,
        RuleToggleResponse,
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Chat", description = "OpenAI-compatible chat completion"),
        (name = "Models", description = "Model listing"),
        (name = "Logs", description = "Transformation logging"),
        (name = "Rules", description = "Rule management"),
    )
)]
pub struct ApiDoc;

/// Run the HTTP server
pub async fn run(
    host: &str,
    port: u16,
    rules_paths: &[String],
    api_key: Option<String>,
) -> anyhow::Result<()> {
    // Initialize rule engine
    let rule_engine = Arc::new(RuleEngine::new_from_paths(rules_paths)?);

    // Start file watcher for hot-reload
    rule_engine.clone().watch_for_changes()?;

    let state = AppState {
        rule_engine,
        api_key,
    };

    // Build router
    let app = Router::new()
        // Dashboard UI
        .route("/", get(handlers::dashboard))
        // API routes
        .route("/health", get(handlers::health))
        .route("/v1/chat/completions", post(handlers::chat_completions))
        .route("/v1/models", get(handlers::list_models))
        .route("/v1/logs", get(handlers::get_logs))
        .route("/v1/logs", delete(handlers::clear_logs))
        .route("/v1/rules", get(handlers::get_rules))
        .route("/v1/rules/:rule_id/toggle", post(handlers::toggle_rule))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Parse address
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    tracing::info!("Listening on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui/", addr);

    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
