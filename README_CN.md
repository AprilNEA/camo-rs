<img src=".github/splash.png" alt="camo-rs" />

[![status](https://img.shields.io/badge/status-stable-blue.svg)](https://github.com/aprilnea/camo-rs/tree/master)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Crates.io Version](https://img.shields.io/crates/v/camo-rs)
<h4><a href="./README.md">English</a> | <strong>简体中文</strong></h4>

## 简介

一个用 Rust 编写的高性能 SSL 图片代理。这是 [Camo](https://github.com/atmos/camo) 的 Rust 实现，参考了 [go-camo](https://github.com/cactus/go-camo)。

Camo 通过 HTTPS 代理不安全的 HTTP 图片，防止在安全页面上出现混合内容警告。

## 功能

- **HMAC-SHA1 URL 签名** - 与原始 Camo 兼容
- **双 URL 格式** - 查询字符串 (`/<digest>?url=...`) 和路径格式 (`/<digest>/<encoded_url>`)
- **双编码支持** - 支持 hex 和 base64 URL 编码
- **内容类型过滤** - 图片类型白名单，可选视频/音频支持
- **大小限制** - 可配置最大内容长度（默认 5MB）
- **重定向跟踪** - 可配置重定向限制（默认 4 次）
- **SSRF 防护** - 屏蔽对私有/内部网络的请求（RFC1918）
- **Prometheus 监控** - 可选 `/metrics` 端点
- **结构化日志** - 使用 tracing 构建

## 安装

### 作为库使用

添加到 `Cargo.toml`：

```toml
# 仅客户端（最小依赖：hmac, sha1, hex, base64）
[dependencies]
camo-rs = "0.1"

# 服务器（包含 tokio, axum, reqwest 等）
[dependencies]
camo = { version="0.1", features=["server"] }
```

### 作为二进制安装
```bash
cargo install camo-rs
```

## Cargo Features

| Feature | 默认 | 说明 |
|---------|------|------|
| `client` | 是 | 核心 URL 签名功能，最小依赖 |
| `server` | 否 | 完整代理服务器，包含 CLI、监控等所有依赖 |
| `worker` | 否 | Cloudflare Workers 支持 |

## Cloudflare Workers

将 camo-rs 部署到 Cloudflare Workers，实现边缘图片代理。

### Fork 部署
1. Fork 此仓库
2. 在 Cloudflare Workers 中部署你 Fork 的仓库

### 一键部署（不推荐）

> [!WARNING]
> Cloudflare 会复制仓库代码而非 Fork，这意味着你将无法获取后续更新。

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/AprilNEA/camo-rs)

### 手动部署

#### 部署

```bash
# 设置密钥
wrangler secret put CAMO_KEY

# 部署
wrangler deploy
```

### 环境变量

| 变量 | 说明 |
|------|------|
| `CAMO_KEY` | HMAC 密钥（使用 `wrangler secret put` 设置） |
| `CAMO_MAX_SIZE` | 最大内容大小（字节），默认 5MB |

## 库使用

```rust
use camo_rs::{CamoUrl, Encoding};

// 使用密钥创建 CamoUrl 生成器
let camo = CamoUrl::new("your-secret-key");

// 签名 URL
let signed = camo.sign("http://example.com/image.png");

// 获取完整的代理 URL
let url = signed.to_url("https://camo.example.com");
// => https://camo.example.com/abc123.../68747470...

// 或只获取路径部分
let path = signed.to_path();
// => /abc123.../68747470...

// 使用 base64 编码代替 hex
let url = camo.sign("http://example.com/image.png")
    .base64()
    .to_url("https://camo.example.com");

// 设置默认编码
let camo = CamoUrl::new("secret").with_encoding(Encoding::Base64);

// 便捷函数
let url = camo::sign_url("secret", "http://example.com/image.png", "https://camo.example.com");

// 验证签名
assert!(camo.verify("http://example.com/image.png", &signed.digest));
```

## 使用

### 启动服务器

**二进制：**

```bash
# 使用环境变量
CAMO_KEY=your-secret-key camo

# 使用命令行参数
camo -k your-secret-key

# 自定义选项
camo -k your-secret-key --listen 0.0.0.0:8081 --max-size 10485760
```

**Docker：**
```bash
docker run
```

### 生成签名 URL

```bash
# 生成 URL 组件
camo -k your-secret sign "https://example.com/image.png"
# 输出:
# Digest: 54cec8e46f18f585268e3972432cd8da7aec6dc1
# Encoded URL: 68747470733a2f2f6578616d706c652e636f6d2f696d6167652e706e67
# Path: /54cec8e46f18f585268e3972432cd8da7aec6dc1/68747470...

# 生成完整 URL
camo -k your-secret sign "https://example.com/image.png" --base "https://camo.example.com"
# 输出: https://camo.example.com/54cec8e46f18f585268e3972432cd8da7aec6dc1/68747470...

# 使用 base64 编码
camo -k your-secret sign "https://example.com/image.png" --base64
```

### URL 格式

代理接受两种 URL 格式：

```
# 路径格式
https://camo.example.com/<digest>/<hex-encoded-url>

# 查询字符串格式
https://camo.example.com/<digest>?url=<url-encoded-url>
```

## 配置

| 选项 | 环境变量 | 默认值 | 说明 |
|------|---------|--------|------|
| `-k, --key` | `CAMO_KEY` | (必需) | URL 签名的 HMAC 密钥 |
| `--listen` | `CAMO_LISTEN` | `0.0.0.0:8080` | 监听地址 |
| `--max-size` | `CAMO_LENGTH_LIMIT` | `5242880` | 最大内容长度（字节） |
| `--max-redirects` | `CAMO_MAX_REDIRECTS` | `4` | 最大重定向次数 |
| `--timeout` | `CAMO_SOCKET_TIMEOUT` | `10` | 套接字超时（秒） |
| `--allow-video` | `CAMO_ALLOW_VIDEO` | `false` | 允许视频类型 |
| `--allow-audio` | `CAMO_ALLOW_AUDIO` | `false` | 允许音频类型 |
| `--block-private` | `CAMO_BLOCK_PRIVATE` | `true` | 屏蔽私有网络（RFC1918） |
| `--metrics` | `CAMO_METRICS` | `false` | 启用 /metrics 端点 |
| `--log-level` | `CAMO_LOG_LEVEL` | `info` | 日志级别 (trace/debug/info/warn/error) |

## 集成

### 在应用中生成 URL

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

## 端点

| 路径 | 说明 |
|------|------|
| `/` | 健康检查，返回 "OK" |
| `/health` | 健康检查，返回 "OK" |
| `/metrics` | Prometheus 指标（如已启用） |
| `/<digest>/<encoded_url>` | 代理端点（路径格式） |
| `/<digest>?url=<url>` | 代理端点（查询格式） |

## 许可证

MIT License

## 致谢

- [atmos/camo](https://github.com/atmos/camo) - 原始 Camo 实现
- [cactus/go-camo](https://github.com/cactus/go-camo) - Go 语言实现参考
