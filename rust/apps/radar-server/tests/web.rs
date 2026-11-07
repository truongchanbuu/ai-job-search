//! Browser authentication and static delivery require no database or credentials.
use radar_adapters::{load_profile, postgres::PgRepository};
use radar_server::{AppState, router};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn static_ui_and_browser_credentials_preserve_api_authorization() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://synthetic@127.0.0.1:1/unused")
        .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let owner = "synthetic-owner-token-for-web-tests-only";
    let reader = "synthetic-reader-token-for-web-tests-only";
    let state = AppState::new(
        PgRepository::new(pool.clone()),
        load_profile(&root.join("rust/fixtures/profiles")).unwrap(),
        owner,
        Some(reader),
    )
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router(state)).await.unwrap();
    });
    let http = Client::new();
    for (path, content_type) in [
        ("/", "text/html"),
        ("/app.css", "text/css"),
        ("/app.js", "text/javascript"),
        ("/radar.png", "image/png"),
    ] {
        let response = http.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with(content_type)
        );
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(response.headers()["referrer-policy"], "no-referrer");
        assert!(
            response.headers()["content-security-policy"]
                .to_str()
                .unwrap()
                .contains("frame-ancestors 'none'")
        );
    }
    assert_eq!(
        http.get(format!("{base}/v1/jobs"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        http.get(format!("{base}/v1/web-access"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    for (credential, can_write) in [(owner, true), (reader, false)] {
        let access: Value = http
            .get(format!("{base}/v1/web-access"))
            .bearer_auth(credential)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let response = http
            .post(format!("{base}/web/connect"))
            .header("x-radar-web", "1")
            .json(&json!({"code":access["code"]}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let set_cookie = response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .to_owned();
        assert!(
            set_cookie.contains("HttpOnly")
                && set_cookie.contains("SameSite=Strict")
                && set_cookie.contains("Max-Age=7200")
        );
        let cookie = set_cookie.split(';').next().unwrap();
        let session: Value = response.json().await.unwrap();
        assert_eq!(session["can_write"], can_write);
        assert!(
            session.get("token").is_none(),
            "Browser credential must not be returned to JavaScript"
        );
        assert_eq!(
            http.post(format!("{base}/web/connect"))
                .header("x-radar-web", "1")
                .json(&json!({"code":access["code"]}))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let resumed = http
            .get(format!("{base}/v1/session"))
            .header("cookie", cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(resumed.status(), StatusCode::OK);
        let value: Value = resumed.json().await.unwrap();
        assert_eq!(value["can_write"], can_write);
        // Ambient cookies cannot authorize cross-origin or missing-CSRF-header writes.
        assert_eq!(
            http.post(format!("{base}/v1/scheduler/scan"))
                .header("cookie", cookie)
                .json(&json!({}))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            http.post(format!("{base}/v1/scheduler/scan"))
                .header("cookie", cookie)
                .header("x-radar-web", "1")
                .header("origin", "https://untrusted.example")
                .json(&json!({}))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        if !can_write {
            for path in [
                "/v1/imports",
                "/v1/scheduler/scan",
                "/v1/preferences/rebuild",
                "/v1/jobs/00000000-0000-0000-0000-000000000000/ai/evaluate",
            ] {
                assert_eq!(
                    http.post(format!("{base}{path}"))
                        .header("cookie", cookie)
                        .header("x-radar-web", "1")
                        .json(&json!({}))
                        .send()
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::FORBIDDEN
                );
            }
        }
        let logout = http
            .post(format!("{base}/web/logout"))
            .header("cookie", cookie)
            .header("x-radar-web", "1")
            .send()
            .await
            .unwrap();
        assert_eq!(logout.status(), StatusCode::NO_CONTENT);
        assert!(
            logout.headers()["set-cookie"]
                .to_str()
                .unwrap()
                .contains("Max-Age=0")
        );
        assert_eq!(
            http.get(format!("{base}/v1/session"))
                .header("cookie", cookie)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        http.get(format!("{base}/v1/session"))
            .bearer_auth(owner)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let invalid = http
        .post(format!("{base}/web/connect"))
        .header("x-radar-web", "1")
        .json(&json!({"code":"invalid"}))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        http.get(format!("{base}/.env"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    server.abort();
    let _ = server.await;
    pool.close().await;
}
