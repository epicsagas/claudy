use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Verify a Slack request signature using HMAC-SHA256.
///
/// Slack signs requests with `v0:{timestamp}:{body}` using the signing secret.
/// The signature header has the format `v0={hex_digest}`.
pub fn verify_signature(
    signing_secret: &str,
    body: &[u8],
    timestamp: &str,
    signature: &str,
) -> bool {
    let sig = match signature.strip_prefix("v0=") {
        Some(s) => s,
        None => return false,
    };

    let base_string = format!("v0:{}:{}", timestamp, String::from_utf8_lossy(body));

    let expected = hmac_sha256(signing_secret.as_bytes(), base_string.as_bytes());
    let expected_hex = hex::encode(expected);

    // Constant-time comparison via hex parsing to avoid timing attacks.
    let Ok(expected_bytes) = hex::decode(sig) else {
        return false;
    };
    let Ok(computed_bytes) = hex::decode(&expected_hex) else {
        return false;
    };

    // Length must match.
    if expected_bytes.len() != computed_bytes.len() {
        return false;
    }

    let mut diff: u8 = 0;
    for (a, b) in expected_bytes.iter().zip(computed_bytes.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Compute HMAC-SHA256 using the standard HMAC construction with SHA-256.
///
/// HMAC(K, m) = SHA256((K' ^ opad) || SHA256((K' ^ ipad) || m))
/// where K' is K padded or hashed to the block size (64 bytes for SHA-256).
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;

    // If key is longer than block size, hash it first.
    let key_block = if key.len() > BLOCK_SIZE {
        let hash = Sha256::digest(key);
        let mut block = [0u8; BLOCK_SIZE];
        block[..32].copy_from_slice(&hash);
        block
    } else {
        let mut block = [0u8; BLOCK_SIZE];
        block[..key.len()].copy_from_slice(key);
        block
    };

    // inner = SHA256((K' ^ ipad) || message)
    let mut inner_hasher = Sha256::new();
    let mut ipad = [0u8; BLOCK_SIZE];
    for (i, k) in key_block.iter().enumerate() {
        ipad[i] = k ^ 0x36;
    }
    inner_hasher.update(ipad);
    inner_hasher.update(message);
    let inner_result = inner_hasher.finalize();

    // outer = SHA256((K' ^ opad) || inner)
    let mut outer_hasher = Sha256::new();
    let mut opad = [0u8; BLOCK_SIZE];
    for (i, k) in key_block.iter().enumerate() {
        opad[i] = k ^ 0x5c;
    }
    outer_hasher.update(opad);
    outer_hasher.update(inner_result);
    let outer_result = outer_hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&outer_result);
    output
}

/// Slack URL verification challenge payload.
#[derive(Debug, Deserialize, Serialize)]
pub struct SlackChallenge {
    pub challenge: String,
    pub token: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Slack event subscription payload (envelope).
#[derive(Debug, Deserialize)]
pub struct SlackEventPayload {
    pub token: String,
    pub team_id: String,
    pub api_app_id: String,
    pub event: serde_json::Value,
    #[serde(rename = "type")]
    pub payload_type: String,
    pub event_id: Option<String>,
    pub event_time: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_signature_valid() {
        // Manually compute a known signature for verification.
        let secret = "test_signing_secret";
        let body = r#"{"type":"url_verification","challenge":"abc123"}"#;
        let timestamp = "1234567890";

        let base_string = format!("v0:{timestamp}:{body}");
        let mac = hmac_sha256(secret.as_bytes(), base_string.as_bytes());
        let signature = format!("v0={}", hex::encode(mac));

        assert!(verify_signature(
            secret,
            body.as_bytes(),
            timestamp,
            &signature
        ));
    }

    #[test]
    fn verify_signature_invalid_body() {
        let secret = "test_signing_secret";
        let body = r#"{"type":"url_verification","challenge":"abc123"}"#;
        let timestamp = "1234567890";

        let base_string = format!("v0:{timestamp}:{body}");
        let mac = hmac_sha256(secret.as_bytes(), base_string.as_bytes());
        let signature = format!("v0={}", hex::encode(mac));

        assert!(!verify_signature(
            secret,
            b"tampered body",
            timestamp,
            &signature
        ));
    }

    #[test]
    fn verify_signature_wrong_prefix() {
        assert!(!verify_signature(
            "secret",
            b"body",
            "123",
            "invalid_prefix=abc"
        ));
    }

    #[test]
    fn verify_signature_malformed_hex() {
        assert!(!verify_signature(
            "secret",
            b"body",
            "123",
            "v0=not_valid_hex!!"
        ));
    }

    #[test]
    fn hmac_sha256_consistency() {
        // Verify our HMAC produces consistent output for the same input.
        let key = b"my_secret";
        let msg = b"hello world";
        let a = hmac_sha256(key, msg);
        let b = hmac_sha256(key, msg);
        assert_eq!(a, b);
    }

    #[test]
    fn hmac_sha256_key_longer_than_block() {
        // Key longer than 64 bytes should be hashed first.
        let key = [0xAB_u8; 100];
        let msg = b"test message";
        let result = hmac_sha256(&key, msg);
        assert_ne!(result, [0u8; 32]);
    }

    #[test]
    fn deserialize_challenge() {
        let json = r#"{"challenge":"abc123","token":"xyz","type":"url_verification"}"#;
        let challenge: SlackChallenge = serde_json::from_str(json).unwrap();
        assert_eq!(challenge.challenge, "abc123");
        assert_eq!(challenge.token, "xyz");
        assert_eq!(challenge.event_type, "url_verification");
    }
}
