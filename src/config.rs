use crate::content_types::{AUDIO_TYPES, IMAGE_TYPES, VIDEO_TYPES};
use clap::{Parser, Subcommand};
use std::sync::Arc;

#[derive(Debug, Clone, Parser)]
#[command(name = "camo-rs", about = "SSL image proxy server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// HMAC key for URL signing
    #[arg(short, long, env = "CAMO_KEY", global = true)]
    pub key: Option<String>,

    /// Listen address
    #[arg(long, env = "CAMO_LISTEN", default_value = "0.0.0.0:8080")]
    pub listen: String,

    /// Maximum content length in bytes
    #[arg(long, env = "CAMO_LENGTH_LIMIT", default_value_t = 5 * 1024 * 1024)]
    pub max_size: u64,

    /// Maximum number of redirects to follow
    #[arg(long, env = "CAMO_MAX_REDIRECTS", default_value_t = 4)]
    pub max_redirects: u32,

    /// Socket timeout in seconds
    #[arg(long, env = "CAMO_SOCKET_TIMEOUT", default_value_t = 10)]
    pub timeout: u64,

    /// Allow video content types
    #[arg(long, env = "CAMO_ALLOW_VIDEO", default_value_t = false)]
    pub allow_video: bool,

    /// Allow audio content types
    #[arg(long, env = "CAMO_ALLOW_AUDIO", default_value_t = false)]
    pub allow_audio: bool,

    /// Block requests to private/internal networks (RFC1918)
    #[arg(long, env = "CAMO_BLOCK_PRIVATE", default_value_t = true)]
    pub block_private: bool,

    /// Enable metrics endpoint at /metrics
    #[arg(long, env = "CAMO_METRICS", default_value_t = false)]
    pub metrics: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "CAMO_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Start the proxy server (default)
    Serve,

    /// Generate a signed URL
    Sign {
        /// The URL to sign
        url: String,

        /// Camo server base URL
        #[arg(long, default_value = "")]
        base: String,

        /// Use base64 encoding instead of hex
        #[arg(long, default_value_t = false)]
        base64: bool,
    },
}

impl Cli {
    pub fn allowed_content_types(&self) -> Vec<&'static str> {
        let mut types: Vec<&'static str> = IMAGE_TYPES.to_vec();

        if self.allow_video {
            types.extend(VIDEO_TYPES);
        }

        if self.allow_audio {
            types.extend(AUDIO_TYPES);
        }

        types
    }
}

pub type SharedConfig = Arc<Cli>;
