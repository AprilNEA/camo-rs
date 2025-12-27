use crate::config::SharedConfig;
use crate::error::{CamoError, Result};
use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue};
use reqwest::Client;
use std::net::IpAddr;
use std::time::Duration;
use url::Url;

pub struct ProxyClient {
    client: Client,
    config: SharedConfig,
}

impl ProxyClient {
    pub fn new(config: SharedConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects as usize))
            .user_agent("camo-rs")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn proxy(&self, url: &str) -> Result<ProxyResponse> {
        let parsed_url = Url::parse(url).map_err(|e| CamoError::InvalidUrl(e.to_string()))?;

        // Validate URL scheme
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(CamoError::InvalidUrl("Only http/https schemes allowed".into()));
        }

        // Block private networks if enabled
        if self.config.block_private {
            self.check_private_network(&parsed_url).await?;
        }

        // Make the request
        let response = self.client.get(url).send().await?;

        // Check content type
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !self.is_allowed_content_type(content_type) {
            return Err(CamoError::ContentTypeNotAllowed(content_type.to_string()));
        }

        // Check content length if present
        if let Some(content_length) = response.content_length() {
            if content_length > self.config.max_size {
                return Err(CamoError::ContentTooLarge(content_length));
            }
        }

        // Build response headers
        let mut headers = HeaderMap::new();

        if let Some(ct) = response.headers().get(header::CONTENT_TYPE) {
            headers.insert(header::CONTENT_TYPE, ct.clone());
        }

        if let Some(cl) = response.headers().get(header::CONTENT_LENGTH) {
            headers.insert(header::CONTENT_LENGTH, cl.clone());
        }

        if let Some(cc) = response.headers().get(header::CACHE_CONTROL) {
            headers.insert(header::CACHE_CONTROL, cc.clone());
        }

        if let Some(etag) = response.headers().get(header::ETAG) {
            headers.insert(header::ETAG, etag.clone());
        }

        if let Some(lm) = response.headers().get(header::LAST_MODIFIED) {
            headers.insert(header::LAST_MODIFIED, lm.clone());
        }

        // Add security headers
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; img-src data:; style-src 'unsafe-inline'"),
        );

        // Stream the response body
        let stream = response.bytes_stream();
        let body = Body::from_stream(stream);

        Ok(ProxyResponse { headers, body })
    }

    fn is_allowed_content_type(&self, content_type: &str) -> bool {
        let ct_lower = content_type.to_lowercase();
        let mime_type = ct_lower.split(';').next().unwrap_or("").trim();

        self.config
            .allowed_content_types()
            .iter()
            .any(|allowed| *allowed == mime_type)
    }

    async fn check_private_network(&self, url: &Url) -> Result<()> {
        let host = url.host_str().ok_or_else(|| CamoError::InvalidUrl("No host".into()))?;

        // Try to resolve the hostname
        let addrs: Vec<IpAddr> = tokio::net::lookup_host(format!(
            "{}:{}",
            host,
            url.port_or_known_default().unwrap_or(80)
        ))
        .await
        .map_err(|e| CamoError::InvalidUrl(e.to_string()))?
        .map(|addr| addr.ip())
        .collect();

        for addr in addrs {
            if is_private_ip(&addr) {
                return Err(CamoError::PrivateNetworkNotAllowed);
            }
        }

        Ok(())
    }
}

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_unspecified()
                // 100.64.0.0/10 (Carrier-grade NAT)
                || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0xC0) == 64)
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback() || ipv6.is_unspecified()
            // Could add more IPv6 private ranges here
        }
    }
}

pub struct ProxyResponse {
    pub headers: HeaderMap,
    pub body: Body,
}
