<img src=".github/splash.png" alt="camo-rs" />

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/aprilnea/camo-rs/tree/master)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Crates.io Version](https://img.shields.io/crates/v/camo-rs)
<h4><strong>English</strong> | <a href="./README_CN.md">简体中文</a></h4>

## Introduction

A high-performance SSL image proxy written in Rust. This is a Rust implementation of [Camo](https://github.com/atmos/camo), inspired by [go-camo](https://github.com/cactus/go-camo).

Camo proxies insecure HTTP images over HTTPS, preventing mixed content warnings on secure pages.

## Features

- **HMAC-SHA1 URL signing** - Compatible with the original Camo
- **Dual URL format** - Query string (`/<digest>?url=...`) and path-based (`/<digest>/<encoded_url>`)
- **Dual encoding** - Supports both hex and base64 URL encoding
- **Content-Type filtering** - Whitelist for image types, optional video/audio support
- **Size limits** - Configurable maximum content length (default 5MB)
- **Redirect following** - Configurable redirect limit (default 4)
- **SSRF protection** - Blocks requests to private/internal networks (RFC1918)
- **Prometheus metrics** - Optional `/metrics` endpoint
- **Structured logging** - Built with tracing

## Installation

### As a library

Add to your `Cargo.toml`:

```toml
# Client only (minimal dependencies: hmac, sha1, hex, base64)
[dependencies]
camo-rs = "0.1"

# Server (includes tokio, axum, reqwest, etc.)
[dependencies]
camo = { version="0.1", features=["server"] }
```

### As a binary
```bash
cargo install camo-rs
```

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `client` | Yes | Core URL signing functionality with minimal dependencies |
| `server` | No | Full proxy server with CLI, metrics, and all dependencies |
| `worker` | No | Cloudflare Workers support |

## Cloudflare Workers

Deploy camo-rs to Cloudflare Workers for edge-based image proxying.

### Fork Deploy
1. Fork this repository
2. Deploy the repository which you forked in Cloudflare Workes.
 
### One-Click Deploy (Not recommended)

> [!WARNING]
> Cloudflare will copy the code from the repository rather than fork it, which will result in losing any updates.
> 
[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/AprilNEA/camo-rs)

### Manual Deployment

#### Deploy

```bash
# Set your secret key
wrangler secret put CAMO_KEY

# Deploy
wrangler deploy
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `CAMO_KEY` | HMAC secret key (use `wrangler secret put`) |
| `CAMO_MAX_SIZE` | Maximum content size in bytes (default: 5MB) |

## Library Usage

```rust
use camo_rs::{CamoUrl, Encoding};

// Create a CamoUrl generator with your secret key
let camo = CamoUrl::new("your-secret-key");

// Sign a URL
let signed = camo.sign("http://example.com/image.png");

// Get the full proxy URL
let url = signed.to_url("https://camo.example.com");
// => https://camo.example.com/abc123.../68747470...

// Or just the path
let path = signed.to_path();
// => /abc123.../68747470...

// Use base64 encoding instead of hex
let url = camo.sign("http://example.com/image.png")
    .base64()
    .to_url("https://camo.example.com");

// Set default encoding
let camo = CamoUrl::new("secret").with_encoding(Encoding::Base64);

// Convenience function
let url = camo::sign_url("secret", "http://example.com/image.png", "https://camo.example.com");

// Verify a digest
assert!(camo.verify("http://example.com/image.png", &signed.digest));
```

## Usage

### Start the server

**Binary:**

```bash
# Using environment variable
CAMO_KEY=your-secret-key camo

# Using CLI argument
camo -k your-secret-key

# With custom options
camo -k your-secret-key --listen 0.0.0.0:8081 --max-size 10485760
```

**Docker:**
```bash
docker run
```

### Generate signed URLs

```bash
# Generate URL components
camo -k your-secret sign "https://example.com/image.png"
# Output:
# Digest: 54cec8e46f18f585268e3972432cd8da7aec6dc1
# Encoded URL: 68747470733a2f2f6578616d706c652e636f6d2f696d6167652e706e67
# Path: /54cec8e46f18f585268e3972432cd8da7aec6dc1/68747470...

# Generate full URL
camo -k your-secret sign "https://example.com/image.png" --base "https://camo.example.com"
# Output: https://camo.example.com/54cec8e46f18f585268e3972432cd8da7aec6dc1/68747470...

# Use base64 encoding
camo -k your-secret sign "https://example.com/image.png" --base64
```

### URL Formats

The proxy accepts two URL formats:

```
# Path format
https://camo.example.com/<digest>/<hex-encoded-url>

# Query string format
https://camo.example.com/<digest>?url=<url-encoded-url>
```

## Configuration

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `-k, --key` | `CAMO_KEY` | (required) | HMAC key for URL signing |
| `--listen` | `CAMO_LISTEN` | `0.0.0.0:8080` | Listen address |
| `--max-size` | `CAMO_LENGTH_LIMIT` | `5242880` | Maximum content length in bytes |
| `--max-redirects` | `CAMO_MAX_REDIRECTS` | `4` | Maximum redirects to follow |
| `--timeout` | `CAMO_SOCKET_TIMEOUT` | `10` | Socket timeout in seconds |
| `--allow-video` | `CAMO_ALLOW_VIDEO` | `false` | Allow video content types |
| `--allow-audio` | `CAMO_ALLOW_AUDIO` | `false` | Allow audio content types |
| `--block-private` | `CAMO_BLOCK_PRIVATE` | `true` | Block private networks (RFC1918) |
| `--metrics` | `CAMO_METRICS` | `false` | Enable /metrics endpoint |
| `--log-level` | `CAMO_LOG_LEVEL` | `info` | Log level (trace/debug/info/warn/error) |

## Integration

### Generate URLs in your application

**JavaScript/Node.js:**

```javascript
const crypto = require('crypto');

function camoUrl(key, url, baseUrl) {
  const digest = crypto.createHmac('sha1', key).update(url).digest('hex');
  const encodedUrl = Buffer.from(url).toString('hex');
  return `${baseUrl}/${digest}/${encodedUrl}`;
}

const url = camoUrl('your-secret', 'http://example.com/image.png', 'https://camo.example.com');
```

**Python:**

```python
import hmac
import hashlib

def camo_url(key: str, url: str, base_url: str) -> str:
    digest = hmac.new(key.encode(), url.encode(), hashlib.sha1).hexdigest()
    encoded_url = url.encode().hex()
    return f"{base_url}/{digest}/{encoded_url}"

url = camo_url('your-secret', 'http://example.com/image.png', 'https://camo.example.com')
```

**Rust:**

```rust
use hmac::{Hmac, Mac};
use sha1::Sha1;

fn camo_url(key: &str, url: &str, base_url: &str) -> String {
    let mut mac = Hmac::<Sha1>::new_from_slice(key.as_bytes()).unwrap();
    mac.update(url.as_bytes());
    let digest = hex::encode(mac.finalize().into_bytes());
    let encoded_url = hex::encode(url.as_bytes());
    format!("{}/{}/{}", base_url, digest, encoded_url)
}
```

## Endpoints

| Path | Description |
|------|-------------|
| `/` | Health check, returns "OK" |
| `/health` | Health check, returns "OK" |
| `/metrics` | Prometheus metrics (if enabled) |
| `/<digest>/<encoded_url>` | Proxy endpoint (path format) |
| `/<digest>?url=<url>` | Proxy endpoint (query format) |

## License

MIT License

## Credits

- [atmos/camo](https://github.com/atmos/camo) - Original Camo implementation
- [cactus/go-camo](https://github.com/cactus/go-camo) - Go implementation reference
