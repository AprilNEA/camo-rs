use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// Decode URL from hex or base64 encoding
///
/// Returns None if decoding fails
///
/// This function is only available with the `server` or `worker` feature.
#[cfg(any(feature = "server", feature = "worker"))]
pub fn decode_url(encoded: &str) -> Option<String> {
    // Try hex first (40+ chars typically)
    if let Ok(bytes) = hex::decode(encoded) {
        if let Ok(s) = String::from_utf8(bytes) {
            return Some(s);
        }
    }

    // Try base64
    if let Ok(bytes) = URL_SAFE_NO_PAD.decode(encoded) {
        if let Ok(s) = String::from_utf8(bytes) {
            return Some(s);
        }
    }

    // Try URL decoding (query string format)
    urlencoding::decode(encoded).ok().map(|s| s.into_owned())
}

/// Encode URL to hex
pub fn encode_url_hex(url: &str) -> String {
    hex::encode(url.as_bytes())
}

/// Encode URL to base64
pub fn encode_url_base64(url: &str) -> String {
    URL_SAFE_NO_PAD.encode(url.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encoding() {
        let url = "https://example.com/image.png";
        let encoded = encode_url_hex(url);
        // Verify it's valid hex
        assert!(hex::decode(&encoded).is_ok());
    }

    #[test]
    fn test_base64_encoding() {
        let url = "https://example.com/image.png";
        let encoded = encode_url_base64(url);
        // Verify it's valid base64
        assert!(URL_SAFE_NO_PAD.decode(&encoded).is_ok());
    }

    #[cfg(any(feature = "server", feature = "worker"))]
    #[test]
    fn test_hex_roundtrip() {
        let url = "https://example.com/image.png";
        let encoded = encode_url_hex(url);
        let decoded = decode_url(&encoded).unwrap();
        assert_eq!(decoded, url);
    }

    #[cfg(any(feature = "server", feature = "worker"))]
    #[test]
    fn test_base64_roundtrip() {
        let url = "https://example.com/image.png";
        let encoded = encode_url_base64(url);
        let decoded = decode_url(&encoded).unwrap();
        assert_eq!(decoded, url);
    }
}
