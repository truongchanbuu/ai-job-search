//! Same-origin static UI and short-lived browser credentials. No business logic.
use super::*;
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct BrowserAccess(Mutex<HashMap<[u8; 32], Grant>>);
struct Grant {
    principal: Principal,
    expires: Instant,
    connect: bool,
}
impl BrowserAccess {
    fn insert(
        &self,
        principal: Principal,
        connect: bool,
        now: Instant,
    ) -> Result<String, ApiError> {
        let mut grants = self.0.lock().expect("browser credential lock");
        grants.retain(|_, grant| grant.expires > now);
        if grants.len() >= 128 {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "browser_limit",
                "Too many browser sessions; try later",
            ));
        }
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        grants.insert(
            Sha256::digest(token.as_bytes()).into(),
            Grant {
                principal,
                expires: now + Duration::from_secs(if connect { 60 } else { 7200 }),
                connect,
            },
        );
        Ok(token)
    }
    pub fn principal(&self, hash: &[u8; 32]) -> Option<Principal> {
        self.0
            .lock()
            .expect("browser credential lock")
            .get(hash)
            .filter(|g| !g.connect && g.expires > Instant::now())
            .map(|g| g.principal)
    }
    fn exchange(&self, code: &str) -> Result<(String, bool), ApiError> {
        let hash: [u8; 32] = Sha256::digest(code.as_bytes()).into();
        let now = Instant::now();
        let principal = {
            let mut grants = self.0.lock().expect("browser credential lock");
            let grant = grants
                .get(&hash)
                .filter(|g| g.connect && g.expires > now)
                .ok_or_else(|| {
                    ApiError::new(
                        StatusCode::UNAUTHORIZED,
                        "expired_link",
                        "Connection link expired or already used; reopen Radar",
                    )
                })?;
            let principal = grant.principal;
            grants.remove(&hash);
            principal
        };
        Ok((self.insert(principal, false, now)?, principal.can_write))
    }
}
pub async fn access(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        serde_json::json!({"code": state.browser.insert(principal, true, Instant::now())?}),
    ))
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connect {
    code: String,
}
pub fn cookie(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == "radar_web_session" && value.len() == 64).then_some(value)
        })
}
/// Browser cookies require an explicit same-origin request header for mutations.
/// Cross-site pages cannot set it without a CORS preflight, which Radar disallows.
pub fn browser_request(headers: &axum::http::HeaderMap, mutation: bool) -> Result<(), ApiError> {
    let origin_ok = match headers.get(header::ORIGIN) {
        None => true,
        Some(origin) => {
            let host = headers
                .get(header::HOST)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            origin
                .to_str()
                .ok()
                .is_some_and(|v| v == format!("http://{host}") || v == format!("https://{host}"))
        }
    };
    if !origin_ok
        || (mutation && headers.get("x-radar-web").and_then(|v| v.to_str().ok()) != Some("1"))
    {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "browser_origin",
            "Browser request must originate from Radar",
        ));
    }
    Ok(())
}
fn session_cookie(token: &str, headers: &axum::http::HeaderMap, clear: bool) -> String {
    let local = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|host| {
            host == "localhost"
                || host.starts_with("localhost:")
                || host == "127.0.0.1"
                || host.starts_with("127.0.0.1:")
                || host == "[::1]"
                || host.starts_with("[::1]:")
        });
    let tls = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("https://"));
    format!(
        "radar_web_session={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{}",
        if clear { 0 } else { 7200 },
        if !local || tls { "; Secure" } else { "" }
    )
}
pub async fn session(Extension(principal): Extension<Principal>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"can_write": principal.can_write}))
}
pub async fn connect(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: Result<Json<Connect>, JsonRejection>,
) -> Result<Response, ApiError> {
    browser_request(&headers, true)?;
    let request = body.map_err(ApiError::json)?.0;
    if request.code.len() != 64 {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "invalid_link",
            "Invalid connection link",
        ));
    }
    let (token, can_write) = state.browser.exchange(&request.code)?;
    if let Some(previous) = cookie(&headers) {
        let hash: [u8; 32] = Sha256::digest(previous.as_bytes()).into();
        state
            .browser
            .0
            .lock()
            .expect("browser credential lock")
            .remove(&hash);
    }
    Ok((
        [(header::SET_COOKIE, session_cookie(&token, &headers, false))],
        Json(serde_json::json!({"can_write": can_write})),
    )
        .into_response())
}
pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
    browser_request(&headers, true)?;
    if let Some(token) = cookie(&headers) {
        let hash: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        state
            .browser
            .0
            .lock()
            .expect("browser credential lock")
            .remove(&hash);
    }
    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, session_cookie("", &headers, true))],
    )
        .into_response())
}
fn asset(content_type: &'static str, body: &'static [u8]) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
            ),
            (header::REFERRER_POLICY, "no-referrer"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
}
pub async fn index() -> impl IntoResponse {
    asset(
        "text/html; charset=utf-8",
        include_bytes!("../web/index.html"),
    )
}
pub async fn css() -> impl IntoResponse {
    asset("text/css; charset=utf-8", include_bytes!("../web/app.css"))
}
pub async fn js() -> impl IntoResponse {
    asset(
        "text/javascript; charset=utf-8",
        include_bytes!("../web/app.js"),
    )
}
pub async fn icon() -> impl IntoResponse {
    asset(
        "image/png",
        include_bytes!("../../../../assets/radar/radar.png"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connect_codes_are_single_use_and_keep_reader_permissions() {
        let access = BrowserAccess::default();
        let code = access
            .insert(
                Principal {
                    actor: "reader",
                    can_write: false,
                },
                true,
                Instant::now(),
            )
            .unwrap();
        assert!(
            access
                .principal(&Sha256::digest(code.as_bytes()).into())
                .is_none()
        );
        let (token, write) = access.exchange(&code).unwrap();
        assert!(!write);
        assert!(access.exchange(&code).is_err());
        assert!(
            !access
                .principal(&Sha256::digest(token.as_bytes()).into())
                .unwrap()
                .can_write
        );
    }
    #[test]
    fn expired_codes_and_sessions_cannot_authorize() {
        let access = BrowserAccess::default();
        let old = Instant::now() - Duration::from_secs(7300);
        let token = access
            .insert(
                Principal {
                    actor: "owner",
                    can_write: true,
                },
                false,
                old,
            )
            .unwrap();
        assert!(
            access
                .principal(&Sha256::digest(token.as_bytes()).into())
                .is_none()
        );
        let code = access
            .insert(
                Principal {
                    actor: "owner",
                    can_write: true,
                },
                true,
                old,
            )
            .unwrap();
        assert!(access.exchange(&code).is_err());
    }
}
