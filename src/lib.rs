//! # camo
//!
//! A Rust library for generating Camo-compatible signed URLs.
//!
//! Camo is an SSL image proxy that routes images through HTTPS to prevent
//! mixed content warnings on secure pages.
//!
//! ## Quick Start
//!
//! ```rust
//! use camo::CamoUrl;
//!
//! let camo = CamoUrl::new("your-secret-key");
//!
//! // Generate a signed URL
//! let signed_url = camo.sign("http://example.com/image.png");
//! println!("{}", signed_url.to_url("https://camo.example.com"));
//! // Output: https://camo.example.com/abc123.../68747470...
//!
//! // Or use the builder pattern
//! let url = camo.sign("http://example.com/image.png")
//!     .base64()
//!     .to_url("https://camo.example.com");
//! ```
//!
//! ## URL Formats
//!
//! The library supports two encoding formats:
//!
//! - **Hex** (default): URL is encoded as hexadecimal
//! - **Base64**: URL is encoded as URL-safe base64
//!
//! Generated URLs follow the format: `<base>/<digest>/<encoded_url>`

#[cfg(all(feature = "server", feature = "worker"))]
compile_error!("Features 'server' and 'worker' are mutually exclusive. Please enable only one.");

mod utils;

#[cfg(any(feature = "server", feature = "worker"))]
pub mod server;

#[cfg(feature = "worker")]
mod worker;
#[cfg(feature = "worker")]
pub use worker::*;

#[cfg(feature = "client")]
mod camo;
#[cfg(feature = "client")]
pub use camo::{CamoUrl, Encoding, SignedUrl};
