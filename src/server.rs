//! HTTP server setup and routing

use crate::handlers;
use crate::models::{
    ChatCompletionRequest, ChatCompletionResponse, Choice, HealthResponse, Message, ModelInfo,
    ModelsResponse, ResponseMessage, RuleInfo, RuleToggleResponse, RulesResponse,
    TransformationLogEntry, TransformationLogResponse, Usage,
};
use crate::rules::RuleEngine;
use axum::{Router, routing::delete, routing::get, routing::post};
use std::net::{SocketAddr, TcpListener};
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

/// Check if a port is available for binding
fn check_port_available(host: &str, port: u16) -> Result<(), String> {
    let addr = format!("{}:{}", host, port);
    match TcpListener::bind(&addr) {
        Ok(_) => Ok(()),
        Err(e) => {
            let hint = if port < 1024 {
                "Ports below 1024 require root/admin privileges."
            } else {
                "Another process may be using this port."
            };
            Err(format!(
                "Port {} is not available on {}: {}\n\nHint: {}\n\nTry:\n  - Use a different port: --port <PORT>\n  - Find what's using the port: lsof -i :{}\n  - Kill the process using the port",
                port, host, e, hint, port
            ))
        },
    }
}

/// Run the HTTP server
pub async fn run(
    host: &str,
    port: u16,
    rules_paths: &[String],
    api_key: Option<String>,
    enable_shell_rules: bool,
) -> anyhow::Result<()> {
    // Check if port is available before doing anything else
    if let Err(msg) = check_port_available(host, port) {
        anyhow::bail!(msg);
    }

    // Initialize rule engine
    let rule_engine = Arc::new(RuleEngine::new_from_paths(rules_paths, enable_shell_rules)?);

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
