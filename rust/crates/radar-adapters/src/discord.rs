mod interactions;
use async_trait::async_trait;
pub use interactions::*;
use radar_application::{bots::*, config::Config};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct DiscordProvider {
    http: reqwest::Client,
    token: String,
    base: String,
    interactive: bool,
}
fn text(value: &str, max: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .take(max)
        .collect()
}
pub fn discord_message(message: &NotificationMessage, id: Uuid) -> Value {
    let mut value = json!({"embeds":[{"title":text(&message.title,256),"description":text(&message.body,1600)}],"allowed_mentions":{"parse":[]}});
    if message.job_id.is_some() && message.job_revision.is_some() {
        value["components"] = json!([{"type":1,"components":[
            {"type":2,"style":1,"label":"Interested","custom_id":format!("radar:{id}:interested")},
            {"type":2,"style":2,"label":"Save","custom_id":format!("radar:{id}:save")},
            {"type":2,"style":2,"label":"Skip","custom_id":format!("radar:{id}:skip")}
        ]}]);
    }
    value
}
impl DiscordProvider {
    pub fn new(token: String, timeout: u64, interactive: bool) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .connect_timeout(std::time::Duration::from_secs(timeout))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "Discord HTTP client could not be configured")?;
        Ok(Self {
            http,
            token,
            base: "https://discord.com/api/v10".into(),
            interactive,
        })
    }
    pub async fn register(
        &self,
        application: &str,
        guild: &str,
        config: &Config,
    ) -> Result<(), String> {
        if !snowflake(application) || !snowflake(guild) {
            return Err("application/guild IDs must be Discord snowflakes".into());
        }
        let descriptions = [
            ("status", "Show Radar scheduler and workspace status"),
            ("jobs", "List current Radar candidates"),
            ("scan", "Queue a bounded mock scan"),
            ("pause", "Pause discovery and ordinary job alerts"),
            ("resume", "Resume the previous active work mode"),
            ("digest", "Queue a current job digest"),
        ];
        let commands: Vec<Value> = descriptions
            .into_iter()
            .filter(|(name, _)| {
                config
                    .notifications
                    .interactions
                    .commands
                    .iter()
                    .any(|c| c == name)
            })
            .map(|(name, description)| json!({"name":name,"description":description,"type":1}))
            .collect();
        let response = self
            .http
            .put(format!(
                "{}/applications/{application}/guilds/{guild}/commands",
                self.base
            ))
            .header("Authorization", format!("Bot {}", self.token))
            .json(&commands)
            .send()
            .await
            .map_err(|_| "Discord command registration connection failed (details redacted)")?;
        if !response.status().is_success() {
            return Err(format!(
                "Discord command registration rejected with HTTP {}",
                response.status().as_u16()
            ));
        }
        Ok(())
    }
}
#[async_trait]
impl NotificationProvider for DiscordProvider {
    async fn deliver(
        &self,
        target: &DeliveryTarget,
        message: &NotificationMessage,
        id: Uuid,
    ) -> Result<DeliveryReceipt, DeliveryFailure> {
        let mut body = discord_message(message, id);
        if !self.interactive {
            body["components"] = json!([]);
        }
        let (request, idempotent) = match target {
            DeliveryTarget::Channel(channel) => {
                if !snowflake(channel) {
                    return Err(DeliveryFailure::new(
                        FailureKind::Permanent,
                        "invalid_channel_id",
                    ));
                }
                body["nonce"] = json!(&id.simple().to_string()[..25]);
                body["enforce_nonce"] = json!(true);
                (
                    self.http
                        .post(format!("{}/channels/{channel}/messages", self.base))
                        .header("Authorization", format!("Bot {}", self.token)),
                    false,
                )
            }
            DeliveryTarget::Interaction {
                application_id,
                token,
            } => {
                if !snowflake(application_id)
                    || token.is_empty()
                    || !token
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                {
                    return Err(DeliveryFailure::new(
                        FailureKind::Permanent,
                        "invalid_interaction_target",
                    ));
                }
                (
                    self.http.patch(format!(
                        "{}/webhooks/{application_id}/{token}/messages/@original",
                        self.base
                    )),
                    true,
                )
            }
        };
        let response = request.json(&body).send().await.map_err(|error| {
            DeliveryFailure::new(
                if error.is_connect() || idempotent {
                    FailureKind::Retryable
                } else {
                    FailureKind::Uncertain
                },
                "discord_transport_failure",
            )
        })?;
        let status = response.status();
        let retry_header = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok());
        let cooldown = if response
            .headers()
            .get("x-ratelimit-remaining")
            .is_some_and(|v| v == "0")
        {
            response
                .headers()
                .get("x-ratelimit-reset-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<f64>().ok())
        } else {
            None
        };
        if status.as_u16() == 429 {
            let body = response.json::<Value>().await.unwrap_or(Value::Null);
            return Err(DeliveryFailure {
                kind: FailureKind::Retryable,
                code: "discord_rate_limited".into(),
                retry_after_seconds: retry_header
                    .into_iter()
                    .chain(body["retry_after"].as_f64())
                    .filter(|s| s.is_finite() && *s >= 0.0)
                    .reduce(f64::max),
                global: body["global"].as_bool() == Some(true),
            });
        }
        if !status.is_success() {
            return Err(DeliveryFailure::new(
                if status.is_server_error() {
                    if idempotent {
                        FailureKind::Retryable
                    } else {
                        FailureKind::Uncertain
                    }
                } else {
                    FailureKind::Permanent
                },
                "discord_http_rejected",
            ));
        }
        let body = response.json::<Value>().await.map_err(|_| {
            DeliveryFailure::new(
                if idempotent {
                    FailureKind::Retryable
                } else {
                    FailureKind::Uncertain
                },
                "discord_response_unknown",
            )
        })?;
        let remote_id = body["id"]
            .as_str()
            .filter(|s| snowflake(s))
            .ok_or_else(|| {
                DeliveryFailure::new(FailureKind::Uncertain, "discord_message_id_unknown")
            })?
            .to_string();
        Ok(DeliveryReceipt {
            remote_id,
            cooldown_seconds: cooldown,
        })
    }
}
pub fn snowflake(value: &str) -> bool {
    !value.is_empty() && value.len() <= 20 && value.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json, Router,
        http::{HeaderMap, StatusCode},
        routing::{patch, post, put},
    };
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    #[tokio::test]
    async fn discord_transport_uses_v10_cards_nonce_mentions_rate_limits_and_original_response_edits()
     {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let count = Arc::new(AtomicUsize::new(0));
        let capture = requests.clone();
        let calls = count.clone();
        let app = Router::new()
            .route(
                "/channels/{id}/messages",
                post(move |headers: HeaderMap, Json(body): Json<Value>| {
                    let capture = capture.clone();
                    let calls = calls.clone();
                    async move {
                        assert_eq!(headers["authorization"], "Bot synthetic-test-token");
                        capture.lock().unwrap().push(body);
                        if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            (
                                StatusCode::TOO_MANY_REQUESTS,
                                [("retry-after", "120")],
                                Json(json!({"retry_after":90.0,"global":true})),
                            )
                                .into_response()
                        } else {
                            (
                                StatusCode::OK,
                                [
                                    ("x-ratelimit-remaining", "0"),
                                    ("x-ratelimit-reset-after", "5.5"),
                                ],
                                Json(json!({"id":"70001"})),
                            )
                                .into_response()
                        }
                    }
                }),
            )
            .route(
                "/webhooks/{application}/{token}/messages/@original",
                patch(|| async { Json(json!({"id":"70002"})) }),
            )
            .route(
                "/applications/{application}/guilds/{guild}/commands",
                put(|Json(commands): Json<Vec<Value>>| async move {
                    assert_eq!(commands.len(), 6);
                    Json(commands)
                }),
            );
        use axum::response::IntoResponse;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut provider = DiscordProvider::new("synthetic-test-token".into(), 2, true).unwrap();
        provider.base = format!("http://{address}");
        let message = NotificationMessage {
            title: "Synthetic job @everyone".into(),
            body: "Source data only".into(),
            job_id: Some(Uuid::new_v4()),
            job_revision: Some(0),
            score: Some(100.0),
        };
        let id = Uuid::new_v4();
        let error = provider
            .deliver(&DeliveryTarget::Channel("30001".into()), &message, id)
            .await
            .unwrap_err();
        assert_eq!(error.kind, FailureKind::Retryable);
        assert_eq!(error.retry_after_seconds, Some(120.0));
        assert!(error.global);
        let receipt = provider
            .deliver(&DeliveryTarget::Channel("30001".into()), &message, id)
            .await
            .unwrap();
        assert_eq!(receipt.remote_id, "70001");
        assert_eq!(receipt.cooldown_seconds, Some(5.5));
        {
            let requests = requests.lock().unwrap();
            assert_eq!(requests[0]["nonce"], requests[1]["nonce"]);
            assert_eq!(requests[0]["enforce_nonce"], true);
            assert_eq!(requests[0]["allowed_mentions"]["parse"], json!([]));
            assert_eq!(
                requests[0]["components"][0]["components"]
                    .as_array()
                    .unwrap()
                    .len(),
                3
            );
        }
        provider.interactive = false;
        provider
            .deliver(
                &DeliveryTarget::Channel("30001".into()),
                &message,
                Uuid::new_v4(),
            )
            .await
            .unwrap();
        assert_eq!(
            requests.lock().unwrap().last().unwrap()["components"],
            json!([])
        );
        provider
            .deliver(
                &DeliveryTarget::Interaction {
                    application_id: "10001".into(),
                    token: "synthetic-interaction-token".into(),
                },
                &NotificationMessage {
                    job_id: None,
                    job_revision: None,
                    ..message
                },
                id,
            )
            .await
            .unwrap();
        mod config_fixture {
            include!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/config.rs"
            ));
        }
        let config = Config::load(config_fixture::directory().path()).unwrap();
        provider.register("10001", "40001", &config).await.unwrap();
        task.abort();
    }
}
