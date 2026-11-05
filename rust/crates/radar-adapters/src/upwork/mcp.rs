use super::*;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    service::{RoleClient, RunningService},
    transport::{
        StreamableHttpClientTransport,
        auth::{AuthClient, AuthorizationManager, AuthorizationRequest, AuthorizationSession},
        streamable_http_client::{
            AuthRequiredError, HttpStatusError, StreamableHttpClientTransportConfig,
        },
    },
};
use std::{error::Error, time::Duration};

#[async_trait::async_trait]
pub trait McpSession: Send {
    async fn tools(&mut self) -> Result<Vec<Value>, UpworkError>;
    async fn call(&mut self, name: &str, args: Value) -> Result<Value, UpworkError>;
}
pub struct OfficialSession {
    service: RunningService<RoleClient, ()>,
    timeout: Duration,
}
impl OfficialSession {
    pub async fn connect(
        settings: &UpworkSettings,
        state: state::SharedState,
    ) -> Result<Self, UpworkError> {
        let timeout = Duration::from_secs(settings.request_timeout_seconds);
        let manager = manager(state, timeout).await?;
        let http = reqwest_mcp::Client::builder()
            .redirect(reqwest_mcp::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|_| UpworkError::new("transport_configuration"))?;
        let client = AuthClient::new(http, manager);
        let mut config = StreamableHttpClientTransportConfig::with_uri(ENDPOINT);
        config.max_concurrent_requests = 1;
        let mut retries = rmcp::transport::common::client_side_sse::FixedInterval::default();
        retries.max_times = Some(0);
        retries.duration = timeout;
        config.retry_config = std::sync::Arc::new(retries);
        config.reinit_on_expired_session = false;
        config.max_sse_event_size = 2 * 1024 * 1024;
        let transport = StreamableHttpClientTransport::with_client(client, config);
        let service = tokio::time::timeout(timeout, ().serve(transport))
            .await
            .map_err(|_| UpworkError::new("timeout"))?
            .map_err(|e| classify(&e))?;
        Ok(Self { service, timeout })
    }
    pub async fn close(self) {
        let _ = self.service.cancel().await;
    }
}
#[async_trait::async_trait]
impl McpSession for OfficialSession {
    async fn tools(&mut self) -> Result<Vec<Value>, UpworkError> {
        let result = tokio::time::timeout(self.timeout, async {
            let mut tools = Vec::new();
            let mut cursor = None;
            let mut seen = std::collections::BTreeSet::new();
            for _ in 0..3 {
                let page = self
                    .service
                    .list_tools(Some(
                        rmcp::model::PaginatedRequestParams::default().with_cursor(cursor),
                    ))
                    .await
                    .map_err(|e| classify(&e))?;
                tools.extend(page.tools);
                if tools.len() > 200 {
                    return Err(UpworkError::new("tool_catalog_too_large"));
                }
                cursor = page.next_cursor;
                if cursor.is_none() {
                    return Ok(tools);
                }
                if !seen.insert(cursor.clone()) {
                    return Err(UpworkError::new("tool_catalog_pagination_cycle"));
                }
            }
            Err(UpworkError::new("tool_catalog_page_limit"))
        })
        .await
        .map_err(|_| UpworkError::new("timeout"))??;
        result
            .into_iter()
            .map(|v| serde_json::to_value(v).map_err(|_| UpworkError::new("invalid_schema")))
            .collect()
    }
    async fn call(&mut self, name: &str, args: Value) -> Result<Value, UpworkError> {
        let args = args
            .as_object()
            .cloned()
            .ok_or_else(|| UpworkError::new("invalid_parameters"))?;
        // One call only: no elicitation loops, model inference or tool execution requested by content.
        let response = tokio::time::timeout(
            self.timeout,
            self.service
                .call_tool_once(CallToolRequestParams::new(name.to_owned()).with_arguments(args)),
        )
        .await
        .map_err(|_| UpworkError::new("timeout"))?
        .map_err(|e| classify(&e))?;
        let rmcp::model::CallToolResponse::Complete(response) = response else {
            return Err(UpworkError::new("interactive_tool_response_unsupported"));
        };
        let mut value = response.structured_content.clone();
        if value.is_none() {
            for content in &response.content {
                if let Some(text) = content.as_text()
                    && let Ok(json) = serde_json::from_str::<Value>(&text.text)
                {
                    value = Some(json);
                    break;
                }
            }
        }
        let value = value.ok_or_else(|| UpworkError::new("invalid_tool_response"))?;
        check_result(&value, response.is_error.unwrap_or(false))?;
        Ok(value)
    }
}
pub fn classify(error: &(dyn Error + 'static)) -> UpworkError {
    use rmcp::{
        service::{ClientInitializeError, ServiceError},
        transport::streamable_http_client::StreamableHttpError,
    };
    if let Some(ClientInitializeError::TransportError { error, .. }) =
        error.downcast_ref::<ClientInitializeError>()
    {
        return classify(error);
    }
    if let Some(ClientInitializeError::LegacyFallbackFailed { fallback, .. }) =
        error.downcast_ref::<ClientInitializeError>()
    {
        return classify(fallback.as_ref());
    }
    if let Some(inner) = error.downcast_ref::<rmcp::transport::DynamicTransportError>() {
        return classify(inner.error.as_ref());
    }
    if let Some(ServiceError::TransportSend(error)) = error.downcast_ref::<ServiceError>() {
        return classify(error);
    }
    if let Some(transport) = error.downcast_ref::<StreamableHttpError<reqwest_mcp::Error>>() {
        match transport {
            StreamableHttpError::AuthRequired(_) => {
                return UpworkError::new("reauthorization_required");
            }
            StreamableHttpError::InsufficientScope(_) => {
                return UpworkError::new("account_restricted");
            }
            StreamableHttpError::Client(e) => return classify(e),
            StreamableHttpError::HttpStatus(e) => return classify(e),
            StreamableHttpError::Auth(e) => return classify_auth(e),
            _ => {}
        }
    }
    let mut current = Some(error);
    while let Some(e) = current {
        if e.downcast_ref::<AuthRequiredError>().is_some() {
            return UpworkError::new("reauthorization_required");
        }
        if let Some(e) = e.downcast_ref::<HttpStatusError>() {
            let delay = e.retry_after.as_deref().and_then(parse_retry_after);
            return UpworkError {
                category: if e.status == 429 {
                    "rate_limited"
                } else {
                    "transport_failure"
                }
                .into(),
                retry_after_seconds: delay,
            };
        }
        if let Some(e) = e.downcast_ref::<reqwest_mcp::Error>()
            && let Some(code) = e.status()
        {
            if code.as_u16() == 401 {
                return UpworkError::new("reauthorization_required");
            }
            if code.as_u16() == 403 {
                return UpworkError::new("account_restricted");
            }
        }
        current = e.source();
    }
    UpworkError::new("transport_failure")
}
fn classify_auth(error: &rmcp::transport::auth::AuthError) -> UpworkError {
    use rmcp::transport::auth::AuthError;
    UpworkError::new(match error {
        AuthError::AuthorizationRequired
        | AuthError::TokenExpired
        | AuthError::TokenRefreshRejected(_) => "reauthorization_required",
        AuthError::CredentialStoreError(_) => "state_unavailable",
        AuthError::InsufficientScope { .. } => "account_restricted",
        AuthError::TokenRefreshFailed(_) | AuthError::HttpError(_) => "transport_failure",
        _ => "oauth_configuration",
    })
}
pub fn parse_retry_after(s: &str) -> Option<u64> {
    if let Ok(n) = s.parse::<u64>() {
        return Some(n);
    }
    chrono::DateTime::parse_from_rfc2822(s)
        .ok()
        .map(|at| (at.timestamp() - chrono::Utc::now().timestamp()).max(0) as u64)
}
pub fn check_result(value: &Value, is_error: bool) -> Result<(), UpworkError> {
    let code = value.get("error_code").and_then(Value::as_str);
    if is_error || value.get("status").and_then(Value::as_str) == Some("error") || code.is_some() {
        let category = match code.unwrap_or("") {
            c if c.contains("RATE_LIMIT") => "rate_limited",
            c if c.contains("AUTH") || c.contains("TOKEN") => "reauthorization_required",
            c if c.contains("PERMISSION") || c.contains("RESTRICT") || c.contains("SUSPEND") => {
                "account_restricted"
            }
            _ => "tool_error",
        };
        return Err(UpworkError {
            category: category.into(),
            retry_after_seconds: value.get("retry_after_seconds").and_then(|v| {
                v.as_u64().or_else(|| {
                    v.as_f64()
                        .filter(|s| s.is_finite() && *s >= 0.0)
                        .map(|s| s.ceil() as u64)
                })
            }),
        });
    }
    Ok(())
}
async fn manager(
    state: state::SharedState,
    timeout: Duration,
) -> Result<AuthorizationManager, UpworkError> {
    let mut manager = AuthorizationManager::new(ENDPOINT)
        .await
        .map_err(|_| UpworkError::new("oauth_discovery_failed"))?;
    let client = reqwest_mcp::Client::builder()
        .timeout(timeout)
        .redirect(reqwest_mcp::redirect::Policy::none())
        .build()
        .map_err(|_| UpworkError::new("transport_configuration"))?;
    manager
        .with_client(client)
        .map_err(|_| UpworkError::new("oauth_configuration"))?;
    manager.set_credential_store((*state).clone());
    if !tokio::time::timeout(timeout, manager.initialize_from_store())
        .await
        .map_err(|_| UpworkError::new("timeout"))?
        .map_err(|e| classify_auth(&e))?
    {
        return Err(UpworkError::new("reauthorization_required"));
    }
    Ok(manager)
}
/// Interactive PKCE flow: binds only loopback; never uses another client's token store.
pub async fn login(
    settings: &UpworkSettings,
    state: state::SharedState,
) -> Result<(), UpworkError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let _guard = state.lock()?;
    let timeout = Duration::from_secs(settings.request_timeout_seconds);
    let mut manager = AuthorizationManager::new(ENDPOINT)
        .await
        .map_err(|_| UpworkError::new("oauth_discovery_failed"))?;
    let http = reqwest_mcp::Client::builder()
        .timeout(timeout)
        .redirect(reqwest_mcp::redirect::Policy::none())
        .build()
        .map_err(|_| UpworkError::new("transport_configuration"))?;
    manager
        .with_client(http)
        .map_err(|_| UpworkError::new("oauth_configuration"))?;
    manager.set_credential_store((*state).clone());
    let resolution = tokio::time::timeout(timeout, manager.resolve_metadata())
        .await
        .map_err(|_| UpworkError::new("timeout"))?
        .map_err(|_| UpworkError::new("oauth_discovery_failed"))?;
    if !resolution.source.is_discovered() {
        return Err(UpworkError::new("oauth_metadata_required"));
    }
    manager.set_metadata(resolution.metadata);
    let listener =
        tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, settings.callback_port))
            .await
            .map_err(|_| UpworkError::new("callback_bind_failed"))?;
    let redirect = format!("http://127.0.0.1:{}/callback", settings.callback_port);
    let mut request = AuthorizationRequest::new(&redirect)
        .with_client_name("Personal Freelance Radar")
        .with_application_type("native");
    if let Ok(id) = std::env::var(&settings.client_id_env)
        && !id.is_empty()
    {
        request = request.with_preregistered_client(id)
    }
    if let Ok(url) = std::env::var(&settings.client_metadata_url_env)
        && !url.is_empty()
    {
        request = request.with_client_metadata_url(url)
    }
    let session = tokio::time::timeout(timeout, AuthorizationSession::new(manager, request))
        .await
        .map_err(|_| UpworkError::new("timeout"))?
        .map_err(|_| UpworkError::new("oauth_client_registration_failed"))?;
    println!(
        "Open this Radar-owned authorization URL in your browser:\n{}\nWaiting for loopback callback. For a remote host, forward the configured callback port over SSH.",
        session.get_authorization_url()
    );
    let deadline =
        tokio::time::Instant::now() + Duration::from_secs(settings.login_timeout_seconds);
    loop {
        let (mut socket, _) = tokio::time::timeout_at(deadline, listener.accept())
            .await
            .map_err(|_| UpworkError::new("login_timeout"))?
            .map_err(|_| UpworkError::new("callback_failed"))?;
        let mut buffer = [0u8; 8192];
        let mut count = 0;
        let header_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while !buffer[..count].windows(4).any(|v| v == b"\r\n\r\n") {
            if count == buffer.len() {
                return Err(UpworkError::new("callback_invalid"));
            }
            let read = tokio::time::timeout_at(header_deadline, socket.read(&mut buffer[count..]))
                .await
                .map_err(|_| UpworkError::new("callback_failed"))?
                .map_err(|_| UpworkError::new("callback_failed"))?;
            if read == 0 {
                return Err(UpworkError::new("callback_failed"));
            }
            count += read;
        }
        let request = std::str::from_utf8(&buffer[..count]).unwrap_or("");
        let path = request
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("GET "))
            .and_then(|line| line.split(' ').next())
            .unwrap_or("");
        let url = url::Url::parse(&format!(
            "http://127.0.0.1:{}{}",
            settings.callback_port, path
        ))
        .map_err(|_| UpworkError::new("callback_invalid"))?;
        if url.path() != "/callback" {
            let _ = socket
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await;
            continue;
        }
        let result = tokio::time::timeout(timeout, session.handle_callback_url(url.as_str()))
            .await
            .map_err(|_| UpworkError::new("timeout"))?;
        let ok = result.is_ok();
        let body = if ok {
            "Radar authorization saved. You may close this tab."
        } else {
            "Radar authorization rejected. Restart login."
        };
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            if ok { "200 OK" } else { "400 Bad Request" },
            body.len(),
            body
        );
        let _ = socket.write_all(response.as_bytes()).await;
        result.map_err(|_| UpworkError::new("oauth_callback_rejected"))?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json, Router,
        http::{HeaderMap, StatusCode},
        response::IntoResponse,
        routing::post,
    };
    #[tokio::test]
    async fn real_sdk_streamable_http_handles_json_protocol_and_structured_errors() {
        for code in [200u16, 401, 403, 429, 503] {
            let app=Router::new().route("/mcp",post(move |Json(body):Json<Value>|async move{
                if code!=200 {
                    let headers=HeaderMap::from_iter([("retry-after".parse().unwrap(),"120".parse().unwrap()),("www-authenticate".parse().unwrap(),"Bearer resource_metadata=\"https://example.invalid/metadata\"".parse().unwrap())]);
                    return (StatusCode::from_u16(code).unwrap(),headers,"").into_response();
                }
                let result=match body["method"].as_str().unwrap_or("") {
                    "initialize"=>json!({"protocolVersion":"2025-03-26","serverInfo":{"name":"synthetic","version":"1"},"capabilities":{"tools":{}}}),
                    "tools/list"=>json!({"tools":[]}),
                    "tools/call"=>json!({"content":[],"structuredContent":{"jobs":[],"empty_reason":"NO_MATCHES"},"isError":false}),
                    _=>return StatusCode::ACCEPTED.into_response(),
                };
                Json(json!({"jsonrpc":"2.0","id":body["id"],"result":result})).into_response()
            }));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
            let transport = StreamableHttpClientTransport::with_client(
                reqwest_mcp::Client::new(),
                StreamableHttpClientTransportConfig::with_uri(format!("http://{addr}/mcp")),
            );
            let result = ().serve(transport).await;
            if code == 200 {
                let service = result.unwrap();
                let mut session = OfficialSession {
                    service,
                    timeout: Duration::from_secs(2),
                };
                assert!(session.tools().await.unwrap().is_empty());
                assert_eq!(
                    session.call("synthetic", json!({})).await.unwrap()["empty_reason"],
                    "NO_MATCHES"
                );
                session.close().await;
            } else {
                let error = match result {
                    Ok(_) => panic!("unexpected handshake success"),
                    Err(e) => classify(&e),
                };
                assert_eq!(
                    error.category,
                    match code {
                        401 => "reauthorization_required",
                        403 => "account_restricted",
                        429 => "rate_limited",
                        _ => "transport_failure",
                    },
                    "HTTP {code}"
                );
                if code == 429 {
                    assert_eq!(error.retry_after_seconds, Some(120))
                }
            }
            task.abort();
        }
        assert!(check_result(&json!({"status":"error","error_code":"SENSITIVE_RATE_LIMIT_EXCEEDED","retry_after_seconds":30}),false).is_err());
        assert!(check_result(&json!({"jobs":[]}), true).is_err());
        assert_eq!(check_result(&json!({"status":"error","error_code":"SENSITIVE_RATE_LIMIT_EXCEEDED","retry_after_seconds":600.5}),false).unwrap_err().retry_after_seconds,Some(601));
    }
    #[tokio::test]
    async fn radar_owned_oauth_store_pkce_csrf_refresh_and_restart_work_with_fake_authorization_server()
     {
        use axum::extract::Form;
        use base64::Engine;
        use rmcp::transport::auth::{CredentialStore, StoredCredentials};
        use sha2::Digest;
        use std::{
            collections::HashMap,
            sync::{Arc, Mutex},
        };
        let observed = Arc::new(Mutex::new(Vec::<HashMap<String, String>>::new()));
        let copy = observed.clone();
        let app=Router::new().route("/token",post(move |Form(form):Form<HashMap<String,String>>|{let copy=copy.clone();async move{
            let token=if form.get("grant_type").map(String::as_str)==Some("refresh_token"){"synthetic-refreshed"}else{"synthetic-original"};
            copy.lock().unwrap().push(form);
            Json(json!({"access_token":token,"token_type":"Bearer","refresh_token":"synthetic-refresh","expires_in":3600}))
        }}));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let temp = tempfile::tempdir().unwrap();
        let store = state::PrivateState::new(temp.path().join("state"), [9; 32]).unwrap();
        let mut manager = AuthorizationManager::new(format!("http://{addr}/mcp"))
            .await
            .unwrap();
        manager.with_client(reqwest_mcp::Client::new()).unwrap();
        manager.set_credential_store(store.clone());
        manager.set_metadata(serde_json::from_value(json!({"issuer":format!("http://{addr}"),"authorization_endpoint":format!("http://{addr}/authorize"),"token_endpoint":format!("http://{addr}/token"),"code_challenge_methods_supported":["S256"]})).unwrap());
        let session = AuthorizationSession::new(
            manager,
            AuthorizationRequest::new("http://127.0.0.1:8787/callback")
                .with_preregistered_client("synthetic-radar-client"),
        )
        .await
        .map_err(|(_, e)| e)
        .unwrap();
        let url = url::Url::parse(session.get_authorization_url()).unwrap();
        let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
        assert_eq!(query["code_challenge_method"], "S256");
        assert!(
            session
                .handle_callback("synthetic-code", "forged-state")
                .await
                .is_err()
        );
        assert!(observed.lock().unwrap().is_empty());
        session
            .handle_callback("synthetic-code", &query["state"])
            .await
            .unwrap();
        let form = observed.lock().unwrap()[0].clone();
        let verifier = &form["code_verifier"];
        assert_eq!(
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(sha2::Sha256::digest(verifier.as_bytes())),
            query["code_challenge"]
        );
        session.auth_manager.refresh_token().await.unwrap();
        let stored: StoredCredentials = store.load().await.unwrap().unwrap();
        let serialized = serde_json::to_string(&stored).unwrap();
        assert!(serialized.contains("synthetic-refreshed"));
        let encrypted = std::fs::read(temp.path().join("state/oauth.enc")).unwrap();
        assert!(!String::from_utf8_lossy(&encrypted).contains("synthetic"));
        let reloaded = state::PrivateState::new(temp.path().join("state"), [9; 32]).unwrap();
        assert!(reloaded.load().await.unwrap().is_some());
        reloaded.clear().await.unwrap();
        assert!(reloaded.load().await.unwrap().is_none());
        task.abort();
    }
}
