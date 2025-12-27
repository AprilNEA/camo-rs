use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// Generate HMAC-SHA1 digest for a URL
pub fn generate_digest(key: &str, url: &str) -> String {
    let mut mac = HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC accepts any key size");
    mac.update(url.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Verify HMAC-SHA1 digest (returns bool)
pub fn verify_digest(key: &str, url: &str, digest: &str) -> bool {
    let expected = generate_digest(key, url);
    constant_time_eq(expected.as_bytes(), digest.as_bytes())
}

/// Constant-time string comparison
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_generation() {
        let key = "test-secret-key";
        let url = "https://example.com/image.png";
        let digest = generate_digest(key, url);

        assert_eq!(digest.len(), 40); // SHA1 produces 20 bytes = 40 hex chars
        assert!(verify_digest(key, url, &digest));
    }

    #[test]
    fn test_hmac_verification_fails() {
        let key = "test-secret-key";
        let url = "https://example.com/image.png";

        assert!(!verify_digest(key, url, "invalid-digest"));
    }
}
