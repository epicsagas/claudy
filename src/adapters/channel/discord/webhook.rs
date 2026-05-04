use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;

/// Verify an incoming Discord webhook request signature.
///
/// Discord signs requests using Ed25519. The signature covers the concatenation
/// of the `X-Discord-Signature-Timestamp` header value and the raw request body.
///
/// Returns `true` when the signature is valid, `false` otherwise.
pub fn verify_discord_signature(
    public_key: &[u8],
    body: &[u8],
    signature: &[u8],
    timestamp: &[u8],
) -> bool {
    let public_key_array: &[u8; 32] = match <&[u8; 32]>::try_from(public_key) {
        Ok(arr) => arr,
        Err(_) => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(public_key_array) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let sig = match Signature::from_slice(signature) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut message = Vec::with_capacity(timestamp.len() + body.len());
    message.extend_from_slice(timestamp);
    message.extend_from_slice(body);

    verifying_key.verify_strict(&message, &sig).is_ok()
}

/// Interaction types sent by Discord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscordInteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
}

impl TryFrom<u8> for DiscordInteractionType {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Ping),
            2 => Ok(Self::ApplicationCommand),
            3 => Ok(Self::MessageComponent),
            other => Err(other),
        }
    }
}

fn deserialize_interaction_type<'de, D>(deserializer: D) -> Result<DiscordInteractionType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = u8::deserialize(deserializer)?;
    DiscordInteractionType::try_from(raw)
        .map_err(|v| serde::de::Error::custom(format!("unknown Discord interaction type: {v}")))
}

/// Incoming Discord interaction payload (partial -- only fields we need).
#[derive(Debug, Deserialize)]
pub struct DiscordInteraction {
    #[serde(rename = "type", deserialize_with = "deserialize_interaction_type")]
    pub interaction_type: DiscordInteractionType,
    pub id: String,
    pub token: String,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub data: Option<DiscordInteractionData>,
}

/// The `data` field of an interaction (varies by type).
#[derive(Debug, Deserialize)]
pub struct DiscordInteractionData {
    pub options: Option<Vec<DiscordOption>>,
    pub custom_id: Option<String>,
    pub component_type: Option<u8>,
}

/// A single slash-command option.
#[derive(Debug, Deserialize)]
pub struct DiscordOption {
    pub name: String,
    pub value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::Rng;

    #[test]
    fn verify_valid_signature() {
        let mut seed = [0u8; 32];
        rand::rng().fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_bytes();

        let timestamp = b"1234567890";
        let body = br#"{"type":1}"#;

        let mut message = Vec::new();
        message.extend_from_slice(timestamp);
        message.extend_from_slice(body);

        let signature = signing_key.sign(&message);
        let sig_bytes = signature.to_bytes();

        assert!(verify_discord_signature(
            &public_key_bytes,
            body,
            &sig_bytes,
            timestamp,
        ));
    }

    #[test]
    fn reject_tampered_body() {
        let mut seed = [0u8; 32];
        rand::rng().fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_bytes();

        let timestamp = b"1234567890";
        let body = br#"{"type":1}"#;

        let mut message = Vec::new();
        message.extend_from_slice(timestamp);
        message.extend_from_slice(body);

        let signature = signing_key.sign(&message);
        let sig_bytes = signature.to_bytes();

        assert!(!verify_discord_signature(
            &public_key_bytes,
            br#"{"type":2}"#,
            &sig_bytes,
            timestamp,
        ));
    }

    #[test]
    fn reject_invalid_public_key() {
        let mut seed = [0u8; 32];
        rand::rng().fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let _verifying_key = signing_key.verifying_key();

        let timestamp = b"1234567890";
        let body = br#"{"type":1}"#;

        let mut message = Vec::new();
        message.extend_from_slice(timestamp);
        message.extend_from_slice(body);

        let signature = signing_key.sign(&message);
        let sig_bytes = signature.to_bytes();

        // Wrong public key
        let mut wrong_seed = [0u8; 32];
        rand::rng().fill_bytes(&mut wrong_seed);
        let wrong_key = SigningKey::from_bytes(&wrong_seed);
        let wrong_bytes = wrong_key.verifying_key().to_bytes();

        assert!(!verify_discord_signature(
            &wrong_bytes,
            body,
            &sig_bytes,
            timestamp,
        ));
    }

    #[test]
    fn deserialize_ping() {
        let json =
            r#"{"type":1,"id":"9","token":"tok","channel_id":null,"user_id":null,"data":null}"#;
        let interaction: DiscordInteraction = serde_json::from_str(json).expect("parse");
        assert_eq!(interaction.interaction_type, DiscordInteractionType::Ping);
    }

    #[test]
    fn deserialize_application_command() {
        let json = r#"{
            "type":2,
            "id":"99",
            "token":"tok",
            "channel_id":"ch1",
            "user_id":"u1",
            "data":{"options":[{"name":"prompt","value":"hello"}]}
        }"#;
        let interaction: DiscordInteraction = serde_json::from_str(json).expect("parse");
        assert_eq!(
            interaction.interaction_type,
            DiscordInteractionType::ApplicationCommand
        );
        let data = interaction.data.expect("data present");
        let opt = data.options.expect("options present");
        assert_eq!(opt[0].name, "prompt");
        assert_eq!(opt[0].value, Some("hello".into()));
    }
}
