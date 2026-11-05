use crate::discord::{DiscordProvider, DiscordVerifier};
use radar_application::{
    api::BotStatus,
    bots::{InteractionProvider, NotificationProvider},
    config::{Config, Provider},
};
use std::{collections::BTreeMap, sync::Arc};

pub trait SecretSource: Send + Sync {
    fn get(&self, name: &str) -> Option<String>;
}
pub struct EnvironmentSecrets;
impl SecretSource for EnvironmentSecrets {
    fn get(&self, name: &str) -> Option<String> {
        let name = name
            .strip_prefix("${")
            .and_then(|s| s.strip_suffix('}'))
            .unwrap_or(name);
        std::env::var(name).ok().filter(|v| !v.is_empty())
    }
}
pub struct BotProviderRegistry {
    pub secrets: Arc<dyn SecretSource>,
    overrides: BTreeMap<String, Arc<dyn NotificationProvider>>,
}
impl Default for BotProviderRegistry {
    fn default() -> Self {
        Self::new(Arc::new(EnvironmentSecrets))
    }
}
impl BotProviderRegistry {
    pub fn new(secrets: Arc<dyn SecretSource>) -> Self {
        Self {
            secrets,
            overrides: BTreeMap::new(),
        }
    }
    /// Test adapters implement the actual async port; production configuration never creates fake bots.
    pub fn with_provider(mut self, name: &str, provider: Arc<dyn NotificationProvider>) -> Self {
        self.overrides.insert(name.into(), provider);
        self
    }
    pub fn notifier(
        &self,
        config: &Config,
        name: &str,
    ) -> Result<Arc<dyn NotificationProvider>, String> {
        match config.notifications.providers.get(name) {
            Some(Provider::Discord {
                enabled: true,
                token_env,
                ..
            }) => {
                if let Some(provider) = self.overrides.get(name) {
                    return Ok(provider.clone());
                }
                let token = self
                    .secrets
                    .get(token_env)
                    .ok_or("bot token environment reference is not configured")?;
                let settings = config
                    .notifications
                    .runtime
                    .as_ref()
                    .ok_or("notification runtime settings missing")?;
                Ok(Arc::new(DiscordProvider::new(
                    token,
                    settings.request_timeout_seconds,
                    self.interactions_ready(config, name),
                )?))
            }
            Some(Provider::Discord { .. }) => Err("provider disabled".into()),
            Some(Provider::Telegram { .. }) => {
                Err("unsupported provider; no Telegram adapter is installed".into())
            }
            None => Err("unknown provider instance".into()),
        }
    }
    pub fn interactions_ready(&self, config: &Config, name: &str) -> bool {
        let Some(Provider::Discord {
            allowed_user_ids_env,
            allowed_guild_ids_env,
            ..
        }) = config.notifications.providers.get(name)
        else {
            return false;
        };
        let Some(user) = self.secrets.get(allowed_user_ids_env) else {
            return false;
        };
        let Some(guild) = self.secrets.get(allowed_guild_ids_env) else {
            return false;
        };
        let Some(channel) = config
            .notifications
            .destinations
            .values()
            .filter(|d| d.provider == name)
            .find_map(|d| self.secrets.get(&d.channel_id_env))
        else {
            return false;
        };
        self.verifier(config, name).is_ok_and(|v| {
            v.authorize(
                user.split(',').next().unwrap_or("").trim(),
                guild.split(',').next().unwrap_or("").trim(),
                &channel,
            )
        }) && crate::discord::TokenCipher::from_config(config, self.secrets.as_ref()).is_ok()
    }
    pub fn verifier(
        &self,
        config: &Config,
        name: &str,
    ) -> Result<Box<dyn InteractionProvider>, String> {
        Ok(Box::new(DiscordVerifier::from_config(
            config,
            name,
            self.secrets.as_ref(),
        )?))
    }
    pub fn statuses(&self, config: &Config) -> Vec<BotStatus> {
        config
            .notifications
            .providers
            .keys()
            .map(|name| match self.notifier(config, name) {
                Ok(_) => BotStatus {
                    instance: name.clone(),
                    configured: true,
                    diagnostic: if self.interactions_ready(config, name) {
                        "notification and interaction settings configured; see delivery history".into()
                    } else {
                        "notifications configured; interactions disabled or missing credentials/allowlists/key".into()
                    },
                },
                Err(error) => BotStatus {
                    instance: name.clone(),
                    configured: false,
                    diagnostic: error,
                },
            })
            .collect()
    }
}
