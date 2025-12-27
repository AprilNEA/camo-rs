use crate::config::SharedConfig;
use crate::crypto::verify_digest;
use crate::encoding::decode_url;
use crate::error::CamoError;
use crate::proxy::ProxyClient;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub struct AppState {
    pub config: SharedConfig,
    pub proxy: ProxyClient,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let mut router = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/favicon.ico", get(favicon))
        // Query string format: /<digest>?url=<url>
        .route("/{digest}", get(proxy_query))
        // Path format: /<digest>/<encoded_url>
        .route("/{digest}/{*encoded_url}", get(proxy_path))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Add metrics endpoint if enabled
    if state.config.metrics {
        router = router.route("/metrics", get(metrics_handler));
    }

    router
}

async fn health_check() -> &'static str {
    "OK"
}

async fn favicon() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn proxy_query(
    State(state): State<Arc<AppState>>,
    Path(digest): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let url = match params.get("url") {
        Some(u) => u.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing url parameter").into_response(),
    };

    proxy_request(&state, &digest, &url).await
}

async fn proxy_path(
    State(state): State<Arc<AppState>>,
    Path((digest, encoded_url)): Path<(String, String)>,
) -> Response {
    let url = match decode_url(&encoded_url) {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Invalid URL encoding").into_response(),
    };

    proxy_request(&state, &digest, &url).await
}

async fn proxy_request(state: &Arc<AppState>, digest: &str, url: &str) -> Response {
    // Record metrics
    if state.config.metrics {
        metrics::counter!("camo_requests_total").increment(1);
    }

    // Verify digest
    let key = state.config.key.as_ref().expect("key must be set");
    if !verify_digest(key, url, digest) {
        if state.config.metrics {
            metrics::counter!("camo_errors_total", "type" => "digest").increment(1);
        }
        return CamoError::DigestMismatch.into_response();
    }

    // Proxy the request
    match state.proxy.proxy(url).await {
        Ok(response) => {
            if state.config.metrics {
                metrics::counter!("camo_success_total").increment(1);
            }
            (response.headers, response.body).into_response()
        }
        Err(e) => {
            if state.config.metrics {
                let error_type = match &e {
                    CamoError::ContentTypeNotAllowed(_) => "content_type",
                    CamoError::ContentTooLarge(_) => "content_size",
                    CamoError::Timeout => "timeout",
                    CamoError::PrivateNetworkNotAllowed => "private_network",
                    _ => "upstream",
                };
                metrics::counter!("camo_errors_total", "type" => error_type).increment(1);
            }
            e.into_response()
        }
    }
}

async fn metrics_handler() -> impl IntoResponse {
    // Prometheus metrics will be rendered by the metrics-exporter-prometheus crate
    // This is a placeholder - actual implementation depends on how metrics recorder is set up
    "# Metrics endpoint\n"
}
