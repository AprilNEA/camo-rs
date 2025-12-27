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
        let mut types = vec![
            "image/bmp",
            "image/cgm",
            "image/g3fax",
            "image/gif",
            "image/ief",
            "image/jp2",
            "image/jpeg",
            "image/jpg",
            "image/pict",
            "image/png",
            "image/prs.btif",
            "image/svg+xml",
            "image/tiff",
            "image/vnd.adobe.photoshop",
            "image/vnd.djvu",
            "image/vnd.dwg",
            "image/vnd.dxf",
            "image/vnd.fastbidsheet",
            "image/vnd.fpx",
            "image/vnd.fst",
            "image/vnd.fujixerox.edmics-mmr",
            "image/vnd.fujixerox.edmics-rlc",
            "image/vnd.microsoft.icon",
            "image/vnd.ms-modi",
            "image/vnd.net-fpx",
            "image/vnd.wap.wbmp",
            "image/vnd.xiff",
            "image/webp",
            "image/x-cmu-raster",
            "image/x-cmx",
            "image/x-icon",
            "image/x-macpaint",
            "image/x-pcx",
            "image/x-pict",
            "image/x-portable-anymap",
            "image/x-portable-bitmap",
            "image/x-portable-graymap",
            "image/x-portable-pixmap",
            "image/x-quicktime",
            "image/x-rgb",
            "image/x-xbitmap",
            "image/x-xpixmap",
            "image/x-xwindowdump",
            "image/avif",
            "image/heic",
            "image/heif",
        ];

        if self.allow_video {
            types.extend([
                "video/mp4",
                "video/webm",
                "video/ogg",
                "video/quicktime",
                "video/x-msvideo",
            ]);
        }

        if self.allow_audio {
            types.extend([
                "audio/mpeg",
                "audio/ogg",
                "audio/wav",
                "audio/webm",
                "audio/flac",
            ]);
        }

        types
    }
}

pub type SharedConfig = Arc<Cli>;
