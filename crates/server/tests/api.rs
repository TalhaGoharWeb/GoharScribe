//! Server tests: auth flow, document CRUD, and isolation between users.

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use goharscribe_server::{AppState, Db, auth::Auth, router};

fn test_state() -> AppState {
    let db = Db::open_memory().expect("in-memory db");
    let auth = Auth::new(b"test-secret-0123456789abcdef-test");
    AppState { db: Arc::new(Mutex::new(db)), auth }
}

async fn post_json(app: &axum::Router, path: &str, body: Value, token: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    if let Some(t) = token {
        req.headers_mut().insert(header::AUTHORIZATION, format!("Bearer {t}").parse().unwrap());
    }
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

async fn get(app: &axum::Router, path: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method("GET").uri(path);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let req = builder.body(Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

async fn signup_login(app: &axum::Router, email: &str) -> String {
    let (st, _) = post_json(app, "/api/auth/signup", json!({"email": email, "password": "password123"}), None).await;
    assert_eq!(st, StatusCode::CREATED);
    let (st, body) = post_json(app, "/api/auth/login", json!({"email": email, "password": "password123"}), None).await;
    assert_eq!(st, StatusCode::OK);
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn health_ok() {
    let app = router(test_state());
    let (st, body) = get(&app, "/api/health", None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn signup_rejects_bad_input() {
    let app = router(test_state());
    let (st, _) = post_json(&app, "/api/auth/signup", json!({"email": "not-an-email", "password": "password123"}), None).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    let (st, _) = post_json(&app, "/api/auth/signup", json!({"email": "a@b.c", "password": "short"}), None).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn signup_rejects_duplicate() {
    let app = router(test_state());
    let (st, _) = post_json(&app, "/api/auth/signup", json!({"email": "dup@x.c", "password": "password123"}), None).await;
    assert_eq!(st, StatusCode::CREATED);
    let (st, _) = post_json(&app, "/api/auth/signup", json!({"email": "dup@x.c", "password": "password123"}), None).await;
    assert_eq!(st, StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let app = router(test_state());
    post_json(&app, "/api/auth/signup", json!({"email": "u@x.c", "password": "password123"}), None).await;
    let (st, _) = post_json(&app, "/api/auth/login", json!({"email": "u@x.c", "password": "wrongpass1"}), None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn docs_require_auth() {
    let app = router(test_state());
    let (st, _) = get(&app, "/api/docs", None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn doc_crud_roundtrip() {
    let app = router(test_state());
    let token = signup_login(&app, "crud@x.c").await;
    // create
    let (st, body) = post_json(&app, "/api/docs", json!({"title": "My Doc"}), Some(&token)).await;
    assert_eq!(st, StatusCode::CREATED);
    let id = body["id"].as_str().unwrap().to_string();
    // get
    let (st, body) = get(&app, &format!("/api/docs/{id}"), Some(&token)).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body["title"], "My Doc");
    // list
    let (st, body) = get(&app, "/api/docs", Some(&token)).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);
    // update
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/docs/{id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(r#"{"title": "Renamed"}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // delete
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/docs/{id}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let (st, _) = get(&app, &format!("/api/docs/{id}"), Some(&token)).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn users_cannot_see_each_others_docs() {
    let app = router(test_state());
    let t1 = signup_login(&app, "one@x.c").await;
    let t2 = signup_login(&app, "two@x.c").await;
    let (_, body) = post_json(&app, "/api/docs", json!({"title": "Secret"}), Some(&t1)).await;
    let id = body["id"].as_str().unwrap().to_string();
    let (st, _) = get(&app, &format!("/api/docs/{id}"), Some(&t2)).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn exec_command_inserts_text() {
    let app = router(test_state());
    let token = signup_login(&app, "exec@x.c").await;
    let (_, body) = post_json(&app, "/api/docs", json!({"title": "Exec"}), Some(&token)).await;
    let id = body["id"].as_str().unwrap().to_string();
    let (st, body) =
        post_json(&app, &format!("/api/docs/{id}/exec"), json!({"command": "text.insert", "params": {"text": "hello"}}), Some(&token)).await;
    assert_eq!(st, StatusCode::OK);
    assert!(body["ok"].as_bool().unwrap());
}

#[tokio::test]
async fn convert_rejects_bad_format() {
    let app = router(test_state());
    let token = signup_login(&app, "conv@x.c").await;
    let (_, body) = post_json(&app, "/api/docs", json!({"title": "Conv"}), Some(&token)).await;
    let id = body["id"].as_str().unwrap().to_string();
    let (st, _) = post_json(&app, &format!("/api/docs/{id}/convert"), json!({"format": "exe"}), Some(&token)).await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
}
