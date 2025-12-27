use super::content_types::{AUDIO_TYPES, IMAGE_TYPES, VIDEO_TYPES};
#[cfg(feature = "server")]
use clap::{Parser, Subcommand};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "server", derive(Parser))]
#[cfg_attr(
    feature = "server",
    command(name = "camo-rs", about = "SSL image proxy server")
)]
pub struct Config {
    #[cfg(feature = "server")]
    #[command(subcommand)]
    pub command: Option<Command>,

    /// HMAC key for URL signing
    #[cfg_attr(feature = "server", arg(short, long, env = "CAMO_KEY", global = true))]
    pub key: Option<String>,

    /// Listen address
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_LISTEN", default_value = "0.0.0.0:8080"))]
    pub listen: String,

    /// Maximum content length in bytes
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_LENGTH_LIMIT", default_value_t = 5 * 1024 * 1024))]
    pub max_size: u64,

    /// Maximum number of redirects to follow
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_MAX_REDIRECTS", default_value_t = 4))]
    pub max_redirects: u32,

    /// Socket timeout in seconds
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_SOCKET_TIMEOUT", default_value_t = 10))]
    pub timeout: u64,

    /// Allow video content types
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_ALLOW_VIDEO", default_value_t = false))]
    pub allow_video: bool,

    /// Allow audio content types
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_ALLOW_AUDIO", default_value_t = false))]
    pub allow_audio: bool,

    /// Block requests to private/internal networks (RFC1918)
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_BLOCK_PRIVATE", default_value_t = true))]
    pub block_private: bool,

    /// Enable metrics endpoint at /metrics
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_METRICS", default_value_t = false))]
    pub metrics: bool,

    /// Log level (trace, debug, info, warn, error)
    #[cfg_attr(feature = "server", arg(long, env = "CAMO_LOG_LEVEL", default_value = "info"))]
    pub log_level: String,
}

#[cfg(feature = "server")]
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

impl Config {
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

    pub fn is_allowed_content_type(&self, content_type: &str) -> bool {
        let ct_lower = content_type.to_lowercase();
        let mime_type = ct_lower.split(';').next().unwrap_or("").trim();

        self.allowed_content_types()
            .iter()
            .any(|allowed| *allowed == mime_type)
    }
}
