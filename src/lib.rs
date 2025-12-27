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

mod crypto;
mod encoding;

pub use crypto::generate_digest;
pub use encoding::{encode_url_base64, encode_url_hex};

#[cfg(feature = "server")]
pub use encoding::decode_url;

/// URL encoding format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Encoding {
    /// Hexadecimal encoding (default, compatible with original Camo)
    #[default]
    Hex,
    /// URL-safe Base64 encoding
    Base64,
}

/// A signed Camo URL ready for use
#[derive(Debug, Clone)]
pub struct SignedUrl {
    /// The original URL that was signed
    pub original_url: String,
    /// The HMAC-SHA1 digest
    pub digest: String,
    /// The encoded URL
    pub encoded_url: String,
    /// The encoding format used
    pub encoding: Encoding,
}

impl SignedUrl {
    /// Generate the full proxy URL with a base URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::CamoUrl;
    ///
    /// let camo = CamoUrl::new("secret");
    /// let url = camo.sign("http://example.com/image.png")
    ///     .to_url("https://camo.example.com");
    /// ```
    pub fn to_url(&self, base: &str) -> String {
        let base = base.trim_end_matches('/');
        format!("{}/{}/{}", base, self.digest, self.encoded_url)
    }

    /// Get just the path portion (without base URL)
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::CamoUrl;
    ///
    /// let camo = CamoUrl::new("secret");
    /// let path = camo.sign("http://example.com/image.png").to_path();
    /// // Returns: /abc123.../68747470...
    /// ```
    pub fn to_path(&self) -> String {
        format!("/{}/{}", self.digest, self.encoded_url)
    }

    /// Switch to Base64 encoding
    pub fn base64(mut self) -> Self {
        if self.encoding != Encoding::Base64 {
            self.encoded_url = encode_url_base64(&self.original_url);
            self.encoding = Encoding::Base64;
        }
        self
    }

    /// Switch to Hex encoding
    pub fn hex(mut self) -> Self {
        if self.encoding != Encoding::Hex {
            self.encoded_url = encode_url_hex(&self.original_url);
            self.encoding = Encoding::Hex;
        }
        self
    }
}

/// Camo URL generator
///
/// Use this struct to generate signed URLs for a Camo proxy.
///
/// # Example
///
/// ```rust
/// use camo::CamoUrl;
///
/// let camo = CamoUrl::new("your-secret-key");
/// let signed = camo.sign("http://example.com/image.png");
/// let url = signed.to_url("https://camo.example.com");
/// ```
#[derive(Debug, Clone)]
pub struct CamoUrl {
    key: String,
    default_encoding: Encoding,
}

impl CamoUrl {
    /// Create a new CamoUrl generator with the given HMAC key
    ///
    /// # Arguments
    ///
    /// * `key` - The HMAC secret key for signing URLs
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::CamoUrl;
    ///
    /// let camo = CamoUrl::new("your-secret-key");
    /// ```
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            default_encoding: Encoding::Hex,
        }
    }

    /// Set the default encoding format for generated URLs
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::{CamoUrl, Encoding};
    ///
    /// let camo = CamoUrl::new("secret")
    ///     .with_encoding(Encoding::Base64);
    /// ```
    pub fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.default_encoding = encoding;
        self
    }

    /// Sign a URL and return a SignedUrl
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to sign (typically an HTTP image URL)
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::CamoUrl;
    ///
    /// let camo = CamoUrl::new("secret");
    /// let signed = camo.sign("http://example.com/image.png");
    ///
    /// // Get the full URL
    /// let url = signed.to_url("https://camo.example.com");
    ///
    /// // Or just the path
    /// let path = camo.sign("http://example.com/image.png").to_path();
    /// ```
    pub fn sign(&self, url: impl AsRef<str>) -> SignedUrl {
        let url = url.as_ref();
        let digest = generate_digest(&self.key, url);
        let encoded_url = match self.default_encoding {
            Encoding::Hex => encode_url_hex(url),
            Encoding::Base64 => encode_url_base64(url),
        };

        SignedUrl {
            original_url: url.to_string(),
            digest,
            encoded_url,
            encoding: self.default_encoding,
        }
    }

    /// Convenience method to sign and generate a full URL in one call
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::CamoUrl;
    ///
    /// let camo = CamoUrl::new("secret");
    /// let url = camo.sign_url("http://example.com/image.png", "https://camo.example.com");
    /// ```
    pub fn sign_url(&self, url: impl AsRef<str>, base: &str) -> String {
        self.sign(url).to_url(base)
    }

    /// Verify a digest matches the expected value for a URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use camo::CamoUrl;
    ///
    /// let camo = CamoUrl::new("secret");
    /// let signed = camo.sign("http://example.com/image.png");
    ///
    /// assert!(camo.verify("http://example.com/image.png", &signed.digest));
    /// assert!(!camo.verify("http://example.com/image.png", "invalid"));
    /// ```
    pub fn verify(&self, url: impl AsRef<str>, digest: &str) -> bool {
        let expected = generate_digest(&self.key, url.as_ref());
        constant_time_eq(expected.as_bytes(), digest.as_bytes())
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Generate a signed Camo URL (convenience function)
///
/// This is a shorthand for creating a CamoUrl and calling sign_url.
///
/// # Arguments
///
/// * `key` - The HMAC secret key
/// * `url` - The URL to sign
/// * `base` - The Camo proxy base URL
///
/// # Example
///
/// ```rust
/// let url = camo::sign_url("secret", "http://example.com/image.png", "https://camo.example.com");
/// ```
pub fn sign_url(key: &str, url: &str, base: &str) -> String {
    CamoUrl::new(key).sign_url(url, base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_url() {
        let camo = CamoUrl::new("test-secret");
        let signed = camo.sign("http://example.com/image.png");

        assert!(!signed.digest.is_empty());
        assert!(!signed.encoded_url.is_empty());
        assert_eq!(signed.encoding, Encoding::Hex);
    }

    #[test]
    fn test_sign_url_base64() {
        let camo = CamoUrl::new("test-secret").with_encoding(Encoding::Base64);
        let signed = camo.sign("http://example.com/image.png");

        assert_eq!(signed.encoding, Encoding::Base64);
    }

    #[test]
    fn test_to_url() {
        let camo = CamoUrl::new("test-secret");
        let url = camo.sign_url("http://example.com/image.png", "https://camo.example.com");

        assert!(url.starts_with("https://camo.example.com/"));
        assert!(url.contains('/'));
    }

    #[test]
    fn test_verify() {
        let camo = CamoUrl::new("test-secret");
        let signed = camo.sign("http://example.com/image.png");

        assert!(camo.verify("http://example.com/image.png", &signed.digest));
        assert!(!camo.verify("http://example.com/image.png", "invalid-digest"));
    }

    #[test]
    fn test_encoding_switch() {
        let camo = CamoUrl::new("test-secret");
        let signed = camo.sign("http://example.com/image.png");
        let hex_encoded = signed.encoded_url.clone();

        let signed = signed.base64();
        assert_ne!(signed.encoded_url, hex_encoded);
        assert_eq!(signed.encoding, Encoding::Base64);

        let signed = signed.hex();
        assert_eq!(signed.encoded_url, hex_encoded);
        assert_eq!(signed.encoding, Encoding::Hex);
    }

    #[test]
    fn test_convenience_function() {
        let url = sign_url("secret", "http://example.com/image.png", "https://camo.example.com");
        assert!(url.starts_with("https://camo.example.com/"));
    }
}
