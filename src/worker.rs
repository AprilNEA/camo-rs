use crate::{decode_url, is_allowed_image_type, CamoUrl};
use worker::{event, Context, Env, Request, Response, ResponseBody, Result};

fn parse_path(path: &str) -> Option<(String, Option<String>)> {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return None;
    }

    let parts: Vec<&str> = path.splitn(2, '/').collect();
    match parts.len() {
        1 => Some((parts[0].to_string(), None)),
        2 => Some((parts[0].to_string(), Some(parts[1].to_string()))),
        _ => None,
    }
}

async fn handle_request(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();

    // Health check endpoints
    if path == "/" || path == "/health" {
        return Response::ok("OK");
    }

    if path == "/favicon.ico" {
        return Response::error("Not Found", 404);
    }

    // Get config from environment
    let key = match env.secret("CAMO_KEY") {
        Ok(s) => s.to_string(),
        Err(_) => return Response::error("CAMO_KEY not set", 500),
    };

    let max_size: u64 = env
        .var("CAMO_MAX_SIZE")
        .map(|v| v.to_string().parse().unwrap_or(5 * 1024 * 1024))
        .unwrap_or(5 * 1024 * 1024);

    // Parse the path to extract digest and encoded URL
    let (digest, encoded_url) = match parse_path(path) {
        Some((d, e)) => (d, e),
        None => return Response::error("Invalid path", 400),
    };

    // Determine the target URL
    let target_url = if let Some(encoded) = encoded_url {
        // Path format: /<digest>/<encoded_url>
        match decode_url(&encoded) {
            Some(u) => u,
            None => return Response::error("Invalid URL encoding", 400),
        }
    } else {
        // Query format: /<digest>?url=<url>
        match url.query_pairs().find(|(k, _)| k == "url") {
            Some((_, v)) => v.into_owned(),
            None => return Response::error("Missing url parameter", 400),
        }
    };

    // Verify digest
    let camo = CamoUrl::new(&key);
    if !camo.verify(&target_url, &digest) {
        return Response::error("Digest mismatch", 400);
    }

    // Validate URL scheme
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return Response::error("Only http/https allowed", 400);
    }

    // Fetch the target URL
    let fetch_request = match Request::new(&target_url, worker::Method::Get) {
        Ok(r) => r,
        Err(e) => return Response::error(format!("Invalid URL: {}", e), 400),
    };

    let mut upstream_response = match worker::Fetch::Request(fetch_request).send().await {
        Ok(r) => r,
        Err(e) => return Response::error(format!("Fetch error: {}", e), 502),
    };

    let status = upstream_response.status_code();
    if !(200..300).contains(&status) {
        return Response::error(format!("Upstream error: {}", status), 502);
    }

    // Check content type
    let content_type = upstream_response
        .headers()
        .get("content-type")
        .ok()
        .flatten()
        .unwrap_or_default();

    if !is_allowed_image_type(&content_type) {
        return Response::error(format!("Content type not allowed: {}", content_type), 415);
    }

    // Check content length
    if let Some(cl) = upstream_response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
    {
        if let Ok(size) = cl.parse::<u64>() {
            if size > max_size {
                return Response::error(format!("Content too large: {} bytes", size), 413);
            }
        }
    }

    // Get body
    let body = match upstream_response.bytes().await {
        Ok(b) => b,
        Err(e) => return Response::error(format!("Body error: {}", e), 502),
    };

    // Build response
    let headers = worker::Headers::new();

    if let Ok(Some(ct)) = upstream_response.headers().get("content-type") {
        let _ = headers.set("content-type", &ct);
    }

    if let Ok(Some(cc)) = upstream_response.headers().get("cache-control") {
        let _ = headers.set("cache-control", &cc);
    }

    if let Ok(Some(etag)) = upstream_response.headers().get("etag") {
        let _ = headers.set("etag", &etag);
    }

    if let Ok(Some(lm)) = upstream_response.headers().get("last-modified") {
        let _ = headers.set("last-modified", &lm);
    }

    // Security headers
    let _ = headers.set("x-content-type-options", "nosniff");
    let _ = headers.set(
        "content-security-policy",
        "default-src 'none'; img-src data:; style-src 'unsafe-inline'",
    );

    Ok(Response::from_body(ResponseBody::Body(body.to_vec()))?
        .with_headers(headers)
        .with_status(200))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    handle_request(req, env).await
}
