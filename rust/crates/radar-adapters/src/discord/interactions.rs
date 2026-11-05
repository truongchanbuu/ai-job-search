use super::snowflake;
use crate::bots::SecretSource;
use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit,
    aead::{Aead, AeadCore, OsRng, Payload},
};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, VerifyingKey};
use radar_application::{
    bots::*,
    config::{Config, Provider},
};
use radar_domain::FeedbackAction;
use serde_json::Value;
use uuid::Uuid;

pub struct DiscordVerifier {
    key: VerifyingKey,
    application: String,
    users: Vec<String>,
    guilds: Vec<String>,
    channels: Vec<String>,
    commands: Vec<String>,
    actions: Vec<String>,
    max_age: i64,
    future_skew: i64,
}
fn list(source: &dyn SecretSource, name: &str) -> Vec<String> {
    source
        .get(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| snowflake(s))
        .map(str::to_owned)
        .collect()
}
impl DiscordVerifier {
    pub fn from_config(
        config: &Config,
        instance: &str,
        secrets: &dyn SecretSource,
    ) -> Result<Self, String> {
        if !config.notifications.interactions.discord_enabled {
            return Err("Discord interactions disabled".into());
        }
        let Some(Provider::Discord {
            enabled: true,
            application_id_env,
            public_key_env,
            allowed_user_ids_env,
            allowed_guild_ids_env,
            ..
        }) = config.notifications.providers.get(instance)
        else {
            return Err("Discord instance disabled or unknown".into());
        };
        let bytes: [u8; 32] = hex::decode(
            secrets
                .get(public_key_env)
                .ok_or("Discord public key missing")?,
        )
        .map_err(|_| "invalid Discord public key")?
        .try_into()
        .map_err(|_| "invalid Discord public key length")?;
        let key = VerifyingKey::from_bytes(&bytes).map_err(|_| "invalid Discord public key")?;
        let application = secrets
            .get(application_id_env)
            .filter(|v| snowflake(v))
            .ok_or("Discord application ID missing or invalid")?;
        let runtime = config
            .notifications
            .runtime
            .as_ref()
            .ok_or("Discord runtime settings missing")?;
        let channels = config
            .notifications
            .destinations
            .values()
            .filter(|d| d.provider == instance)
            .filter_map(|d| secrets.get(&d.channel_id_env))
            .filter(|v| snowflake(v))
            .collect();
        Ok(Self {
            key,
            application,
            users: list(secrets, allowed_user_ids_env),
            guilds: list(secrets, allowed_guild_ids_env),
            channels,
            commands: config.notifications.interactions.commands.clone(),
            actions: config.notifications.interactions.feedback_actions.clone(),
            max_age: runtime.signature_max_age_seconds,
            future_skew: runtime.signature_future_skew_seconds,
        })
    }
}
impl InteractionProvider for DiscordVerifier {
    fn authorize(&self, user: &str, guild: &str, channel: &str) -> bool {
        self.users.iter().any(|v| v == user)
            && self.guilds.iter().any(|v| v == guild)
            && self.channels.iter().any(|v| v == channel)
    }
    fn verify(
        &self,
        body: &[u8],
        signature: &str,
        timestamp: &str,
        now: DateTime<Utc>,
    ) -> Result<VerifiedInteraction, String> {
        if body.len() > 65536 {
            return Err("interaction too large".into());
        }
        let time = timestamp
            .parse::<i64>()
            .map_err(|_| "invalid signature timestamp")?;
        let age = now.timestamp().saturating_sub(time);
        if age > self.max_age || age < -self.future_skew {
            return Err("signature timestamp outside allowed window".into());
        }
        let signature = Signature::from_slice(
            &hex::decode(signature).map_err(|_| "invalid signature encoding")?,
        )
        .map_err(|_| "invalid signature length")?;
        let signed = [timestamp.as_bytes(), body].concat();
        self.key
            .verify_strict(&signed, &signature)
            .map_err(|_| "invalid Discord signature")?;
        let value: Value = serde_json::from_slice(body).map_err(|_| "invalid Discord JSON")?;
        if value["application_id"].as_str() != Some(&self.application) {
            return Err("Discord application mismatch".into());
        }
        let id = value["id"]
            .as_str()
            .filter(|s| snowflake(s))
            .ok_or("invalid interaction ID")?
            .to_string();
        if value["type"].as_u64() == Some(1) {
            return Ok(VerifiedInteraction {
                id,
                application_id: self.application.clone(),
                user_id: String::new(),
                guild_id: String::new(),
                message_id: None,
                channel_id: String::new(),
                token: String::new(),
                action: None,
                ping: true,
            });
        }
        let user = value["member"]["user"]["id"]
            .as_str()
            .ok_or("guild user required")?;
        let guild = value["guild_id"].as_str().ok_or("guild required")?;
        let channel = value["channel_id"].as_str().ok_or("channel required")?;
        if !self.authorize(user, guild, channel) {
            return Err("Discord sender, guild or channel not authorized".into());
        }
        let token = value["token"]
            .as_str()
            .filter(|s| {
                !s.is_empty()
                    && s.len() <= 512
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            })
            .ok_or("invalid interaction token")?
            .to_string();
        let action = match value["type"].as_u64() {
            Some(2) => {
                let command = value["data"]["name"].as_str().ok_or("missing command")?;
                if !self.commands.iter().any(|v| v == command) {
                    return Err("command disabled".into());
                }
                match command {
                    "status" => BotAction::Status,
                    "jobs" => BotAction::Jobs,
                    "scan" => BotAction::Scan,
                    "pause" => BotAction::Pause,
                    "resume" => BotAction::Resume,
                    "digest" => BotAction::Digest,
                    _ => return Err("command not supported".into()),
                }
            }
            Some(kind @ (3 | 5)) => {
                let custom = value["data"]["custom_id"]
                    .as_str()
                    .ok_or("missing component ID")?;
                let parts: Vec<_> = custom.split(':').collect();
                if parts.len() != 3
                    || parts[0] != (if kind == 3 { "radar" } else { "radar-feedback" })
                {
                    return Err("invalid component ID".into());
                }
                let delivery_id =
                    Uuid::parse_str(parts[1]).map_err(|_| "invalid delivery reference")?;
                if !self.actions.iter().any(|v| v == parts[2]) {
                    return Err("feedback action disabled".into());
                }
                let action = match parts[2] {
                    "interested" => FeedbackAction::Interested,
                    "save" => FeedbackAction::Save,
                    "skip" => FeedbackAction::Skip,
                    _ => return Err("feedback action not supported".into()),
                };
                if kind == 3 && action == FeedbackAction::Skip {
                    BotAction::FeedbackModal {
                        delivery_id,
                        action,
                    }
                } else {
                    let reason = if kind == 5 {
                        value["data"]["components"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .flat_map(|row| row["components"].as_array().into_iter().flatten())
                            .find(|input| input["custom_id"] == "reason")
                            .and_then(|input| input["value"].as_str())
                            .filter(|v| !v.trim().is_empty())
                            .map(str::to_owned)
                    } else {
                        None
                    };
                    if reason.as_ref().is_some_and(|s| s.len() > 2000) {
                        return Err("feedback reason too long".into());
                    }
                    BotAction::Feedback {
                        delivery_id,
                        action,
                        reason,
                    }
                }
            }
            _ => return Err("unsupported interaction type".into()),
        };
        Ok(VerifiedInteraction {
            id,
            application_id: self.application.clone(),
            user_id: user.into(),
            guild_id: guild.into(),
            message_id: value["message"]["id"].as_str().map(str::to_owned),
            channel_id: channel.into(),
            token,
            action: Some(action),
            ping: false,
        })
    }
}

pub struct TokenCipher(ChaCha20Poly1305);
impl TokenCipher {
    pub fn from_config(config: &Config, secrets: &dyn SecretSource) -> Result<Self, String> {
        let name = &config
            .notifications
            .runtime
            .as_ref()
            .ok_or("bot settings missing")?
            .interaction_encryption_key_env;
        let key = hex::decode(
            secrets
                .get(name)
                .ok_or("interaction encryption key missing")?,
        )
        .map_err(|_| "invalid interaction encryption key")?;
        Ok(Self(ChaCha20Poly1305::new_from_slice(&key).map_err(
            |_| "interaction encryption key must be 32 bytes",
        )?))
    }
    pub fn seal(&self, provider: &str, id: &str, token: &str) -> Result<Vec<u8>, String> {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let aad = format!("{provider}:{id}");
        let encrypted = self
            .0
            .encrypt(
                &nonce,
                Payload {
                    msg: token.as_bytes(),
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| "interaction encryption failed")?;
        Ok([nonce.as_slice(), encrypted.as_slice()].concat())
    }
    pub fn open(&self, provider: &str, id: &str, bytes: &[u8]) -> Result<String, String> {
        if bytes.len() < 12 {
            return Err("invalid encrypted interaction token".into());
        }
        let aad = format!("{provider}:{id}");
        let value = self
            .0
            .decrypt(
                bytes[..12].into(),
                Payload {
                    msg: &bytes[12..],
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| "interaction token decryption failed")?;
        String::from_utf8(value).map_err(|_| "invalid decrypted token".into())
    }
}
