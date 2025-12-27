use crate::error::{CamoError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// Decode URL from hex or base64 encoding
pub fn decode_url(encoded: &str) -> Result<String> {
    // Try hex first (40+ chars typically)
    if let Ok(bytes) = hex::decode(encoded) {
        return String::from_utf8(bytes).map_err(|_| CamoError::InvalidUrlEncoding);
    }

    // Try base64
    if let Ok(bytes) = URL_SAFE_NO_PAD.decode(encoded) {
        return String::from_utf8(bytes).map_err(|_| CamoError::InvalidUrlEncoding);
    }

    // Try URL decoding (query string format)
    urlencoding::decode(encoded)
        .map(|s| s.into_owned())
        .map_err(|_| CamoError::InvalidUrlEncoding)
}

/// Encode URL to hex
#[allow(dead_code)]
pub fn encode_url_hex(url: &str) -> String {
    hex::encode(url.as_bytes())
}

/// Encode URL to base64
#[allow(dead_code)]
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
        let decoded = decode_url(&encoded).unwrap();
        assert_eq!(decoded, url);
    }

    #[test]
    fn test_base64_encoding() {
        let url = "https://example.com/image.png";
        let encoded = encode_url_base64(url);
        let decoded = decode_url(&encoded).unwrap();
        assert_eq!(decoded, url);
    }
}
