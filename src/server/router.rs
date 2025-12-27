use super::config::Config;
use super::error::CamoError;

use crate::utils::crypto::verify_digest;
use crate::utils::encoding::decode_url;

#[cfg(feature = "server")]
use crate::server::http_client::ReqwestClient;

#[cfg(feature = "worker")]
use crate::server::http_client::WorkerFetchClient;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn from_config(config: &Config) -> Self {
        AppState {
            config: config.clone(),
        }
    }
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
        .with_state(state.clone());

    #[cfg(feature = "worker")]
    {
        router = router.layer(Extension(WorkerFetchClient::new(&state.config)));
        return router;
    }

    #[cfg(feature = "server")]
    {
        // Add metrics endpoint if enabled
        if state.config.metrics {
            router = router.route("/metrics", get(metrics_handler));
        }
        router = router.layer(Extension(ReqwestClient::new(&state.config)));
        return router.layer(tower_http::trace::TraceLayer::new_for_http());
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn favicon() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn proxy_query(
    Path(digest): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    #[cfg(feature = "worker")] Extension(http_client): Extension<WorkerFetchClient>,
    #[cfg(feature = "server")] Extension(http_client): Extension<ReqwestClient>,
) -> Response {
    let url = match params.get("url") {
        Some(u) => u.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing url parameter").into_response(),
    };

    proxy_request(&state, &digest, &url, &http_client).await
}

async fn proxy_path(
    Path((digest, encoded_url)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    #[cfg(feature = "worker")] Extension(http_client): Extension<WorkerFetchClient>,
    #[cfg(feature = "server")] Extension(http_client): Extension<ReqwestClient>,
) -> Response {
    let url = match decode_url(&encoded_url) {
        Some(u) => u,
        None => return (StatusCode::BAD_REQUEST, "Invalid URL encoding").into_response(),
    };

    proxy_request(&state, &digest, &url, &http_client).await
}

async fn proxy_request(
    state: &Arc<AppState>,
    digest: &str,
    url: &str,
    #[cfg(feature = "worker")] http_client: &WorkerFetchClient,
    #[cfg(feature = "server")] http_client: &ReqwestClient,
) -> Response {
    // Record metrics
    // #[cfg(feature = "metrics")]
    // if state.config.metrics {
    //     metrics::counter!("camo_requests_total").increment(1);
    // }

    // Verify digest
    let key = state.config.key.as_ref().expect("key must be set");
    if !verify_digest(key, url, digest) {
        // #[cfg(feature = "metrics")]
        // if state.config.metrics {
        //     metrics::counter!("camo_errors_total", "type" => "digest").increment(1);
        // }
        return CamoError::DigestMismatch.into_response();
    }

    let url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => {
            // #[cfg(feature = "metrics")]
            // if state.config.metrics {
            //     metrics::counter!("camo_errors_total", "type" => "url_parse").increment(1);
            // }
            return CamoError::InvalidUrl("Malformed URL".into()).into_response();
        }
    };

    // Validate URL scheme
    if url.scheme() != "http" && url.scheme() != "https" {
        return CamoError::InvalidUrl("Only http/https schemes allowed".into()).into_response();
    }

    // Proxy the request
    match http_client.get(url).await {
        Ok(response) => {
            // #[cfg(feature = "metrics")]
            // if state.config.metrics {
            //     metrics::counter!("camo_success_total").increment(1);
            // }
            response.into_response()
        }
        Err(e) => {
            if state.config.metrics {
                let _error_type = match &e {
                    CamoError::ContentTypeNotAllowed(_) => "content_type",
                    CamoError::ContentTooLarge(_) => "content_size",
                    CamoError::Timeout => "timeout",
                    CamoError::PrivateNetworkNotAllowed => "private_network",
                    _ => "upstream",
                };
                // #[cfg(feature = "metrics")]
                // metrics::counter!("camo_errors_total", "type" => error_type).increment(1);
            }
            e.into_response()
        }
    }
}

#[allow(dead_code)]
async fn metrics_handler() -> impl IntoResponse {
    // Prometheus metrics will be rendered by the metrics-exporter-prometheus crate
    // This is a placeholder - actual implementation depends on how metrics recorder is set up
    "# Metrics endpoint\n"
}
