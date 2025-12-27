use super::super::{
    config::Config,
    error::{CamoError, Result},
};
use axum::http;
use http::{HeaderMap, HeaderValue};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use url::Url;
use worker::{Fetch, Method, RequestInit};

/// A wrapper that marks a future as Send.
/// SAFETY: Only use in single-threaded environments like Cloudflare Workers.
#[pin_project::pin_project]
struct UnsafeSendFuture<F>(#[pin] F);

// SAFETY: Cloudflare Workers are single-threaded, so this is safe
unsafe impl<F> Send for UnsafeSendFuture<F> {}

impl<F: Future> Future for UnsafeSendFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().0.poll(cx)
    }
}

#[derive(Clone)]
pub struct WorkerFetchClient {
    pub config: Config,
}

#[derive(Clone)]
pub struct WorkerFetchResponse {
    pub body: Vec<u8>,
    pub headers: HeaderMap,
}

impl WorkerFetchClient {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Returns a Send-safe future for use with axum.
    /// SAFETY: This is safe because Cloudflare Workers are single-threaded.
    pub fn get(&self, url: Url) -> impl Future<Output = Result<WorkerFetchResponse>> + Send {
        let config = self.config.clone();

        UnsafeSendFuture(async move {
            let mut init = RequestInit::new();
            init.with_method(Method::Get);
            let request = worker::Request::new_with_init(&url.to_string(), &init)
                .map_err(|e| CamoError::InvalidUrl(e.to_string()))?;

            let mut response = Fetch::Request(request)
                .send()
                .await
                .map_err(|e| CamoError::Upstream(e.to_string()))?;

            // Check content type
            let content_type = response
                .headers()
                .get("content-type")
                .ok()
                .flatten()
                .unwrap_or_default();

            if !config.is_allowed_content_type(&content_type) {
                return Err(CamoError::ContentTypeNotAllowed(content_type.to_string()));
            }

            // Check content length if present
            if let Ok(Some(cl_str)) = response.headers().get("content-length") {
                if let Ok(content_length) = cl_str.parse::<u64>() {
                    if content_length > config.max_size {
                        return Err(CamoError::ContentTooLarge(content_length));
                    }
                }
            }

            // Extract headers before consuming response
            let resp_content_type = response.headers().get("content-type").ok().flatten();
            let resp_cache_control = response.headers().get("cache-control").ok().flatten();
            let resp_etag = response.headers().get("etag").ok().flatten();
            let resp_last_modified = response.headers().get("last-modified").ok().flatten();

            // Get response body
            let body = response
                .bytes()
                .await
                .map_err(|e| CamoError::Upstream(e.to_string()))?;

            // Check actual body size
            if body.len() as u64 > config.max_size {
                return Err(CamoError::ContentTooLarge(body.len() as u64));
            }

            // Build response headers using http::HeaderMap (Send-safe)
            let mut headers = HeaderMap::new();

            if let Some(ct) = resp_content_type {
                if let Ok(v) = HeaderValue::from_str(&ct) {
                    headers.insert(http::header::CONTENT_TYPE, v);
                }
            }

            if let Some(cc) = resp_cache_control {
                if let Ok(v) = HeaderValue::from_str(&cc) {
                    headers.insert(http::header::CACHE_CONTROL, v);
                }
            }

            if let Some(etag) = resp_etag {
                if let Ok(v) = HeaderValue::from_str(&etag) {
                    headers.insert(http::header::ETAG, v);
                }
            }

            if let Some(lm) = resp_last_modified {
                if let Ok(v) = HeaderValue::from_str(&lm) {
                    headers.insert(http::header::LAST_MODIFIED, v);
                }
            }

            // Add security headers
            headers.insert(
                http::header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                http::header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(
                    "default-src 'none'; img-src data:; style-src 'unsafe-inline'",
                ),
            );
            headers.insert(
                http::header::CONTENT_LENGTH,
                HeaderValue::from_str(&body.len().to_string()).unwrap(),
            );

            Ok(WorkerFetchResponse { body, headers })
        })
    }
}

impl axum::response::IntoResponse for WorkerFetchResponse {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        let mut response = axum::http::Response::builder()
            .status(200)
            .body(axum::body::Body::from(self.body))
            .unwrap();

        *response.headers_mut() = self.headers;

        response
    }
}
