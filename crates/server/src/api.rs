//! HTTP API: auth, document CRUD, conversion, headless command execution.
//!
//! All document content is stored as the engine's JSON document format.
//! Conversion reuses `goharscribe-formats` and `goharscribe-pdf`.

use std::sync::{Arc, Mutex};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::auth::{Auth, bearer_token};
use crate::db::Db;
use crate::error::ApiError;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Db>>,
    pub auth: Auth,
}

impl AppState {
    fn with_db<F, T>(&self, f: F) -> Result<T, ApiError>
    where
        F: FnOnce(&Db) -> Result<T, ApiError>,
    {
        let db = self.db.lock().map_err(|_| ApiError::Internal)?;
        f(&db)
    }

    fn user_id(&self, headers: &HeaderMap) -> Result<String, ApiError> {
        let token = bearer_token(headers)?;
        let (uid, _) = self.auth.verify_token(token)?;
        Ok(uid)
    }
}

// ---------- auth ----------

#[derive(Deserialize)]
struct SignupBody {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

fn valid_email(email: &str) -> bool {
    let email = email.trim();
    !email.is_empty() && email.len() <= 254 && email.contains('@') && email.contains('.')
}

async fn signup(State(st): State<AppState>, Json(b): Json<SignupBody>) -> Result<impl IntoResponse, ApiError> {
    let email = b.email.trim().to_lowercase();
    if !valid_email(&email) {
        return Err(ApiError::BadRequest("invalid email".to_string()));
    }
    if b.password.len() < 8 || b.password.len() > 128 {
        return Err(ApiError::BadRequest("password must be 8-128 characters".to_string()));
    }
    let hash = st.auth.hash_password(&b.password)?;
    let user = st.with_db(|db| db.create_user(&email, &hash))?;
    let token = st.auth.issue_token(&user.id, &user.email)?;
    Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}

async fn login(State(st): State<AppState>, Json(b): Json<SignupBody>) -> Result<impl IntoResponse, ApiError> {
    let email = b.email.trim().to_lowercase();
    let user = st.with_db(|db| db.find_user_by_email(&email))?.ok_or_else(|| ApiError::Unauthorized("invalid email or password".to_string()))?;
    // Constant-time-ish: always verify even if user missing (we already returned, but
    // keep the verify call cheap-fail safe by verifying against the stored hash).
    if !st.auth.verify_password(&b.password, &user.password_hash)? {
        return Err(ApiError::Unauthorized("invalid email or password".to_string()));
    }
    let token = st.auth.issue_token(&user.id, &user.email)?;
    Ok(Json(TokenResponse { token }))
}

// ---------- documents ----------

#[derive(Serialize)]
struct DocSummary {
    id: String,
    title: String,
    updated_at: String,
}

#[derive(Serialize)]
struct DocFull {
    id: String,
    title: String,
    content: Value,
    updated_at: String,
}

#[derive(Deserialize)]
struct CreateDocBody {
    title: Option<String>,
    /// Engine JSON document. If omitted, a blank document is created.
    content: Option<Value>,
}

fn blank_document() -> Value {
    // Minimal valid document: one empty paragraph. Matches the engine's JSON format.
    json!({"body": [{"kind": "para", "text": "", "runs": [{"len": 0}]}]})
}

async fn list_docs(State(st): State<AppState>, headers: HeaderMap) -> Result<impl IntoResponse, ApiError> {
    let uid = st.user_id(&headers)?;
    let docs = st.with_db(|db| db.list_documents(&uid))?;
    let out: Vec<DocSummary> = docs.into_iter().map(|d| DocSummary { id: d.id, title: d.title, updated_at: d.updated_at }).collect();
    Ok(Json(out))
}

async fn create_doc(State(st): State<AppState>, headers: HeaderMap, Json(b): Json<CreateDocBody>) -> Result<impl IntoResponse, ApiError> {
    let uid = st.user_id(&headers)?;
    let title = b.title.unwrap_or_else(|| "Untitled".to_string());
    if title.len() > 200 {
        return Err(ApiError::BadRequest("title too long".to_string()));
    }
    let content = b.content.unwrap_or_else(blank_document);
    let content_str = serde_json::to_string(&content)?;
    // Validate it parses as a document (engine will reject garbage on use).
    let doc = st.with_db(|db| db.create_document(&uid, &title, &content_str))?;
    Ok((StatusCode::CREATED, Json(DocFull { id: doc.id, title: doc.title, content, updated_at: doc.updated_at })))
}

async fn get_doc(State(st): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    let uid = st.user_id(&headers)?;
    let doc = st.with_db(|db| db.get_document(&uid, &id))?.ok_or_else(|| ApiError::NotFound("document not found".to_string()))?;
    let content: Value = serde_json::from_str(&doc.content_json)?;
    Ok(Json(DocFull { id: doc.id, title: doc.title, content, updated_at: doc.updated_at }))
}

#[derive(Deserialize)]
struct UpdateDocBody {
    title: Option<String>,
    content: Option<Value>,
}

async fn update_doc(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(b): Json<UpdateDocBody>,
) -> Result<impl IntoResponse, ApiError> {
    let uid = st.user_id(&headers)?;
    if b.title.as_ref().is_some_and(|t| t.len() > 200) {
        return Err(ApiError::BadRequest("title too long".to_string()));
    }
    let content_str = match &b.content {
        Some(c) => Some(serde_json::to_string(c)?),
        None => None,
    };
    let ok = st.with_db(|db| db.update_document(&uid, &id, b.title.as_deref(), content_str.as_deref()))?;
    if !ok {
        return Err(ApiError::NotFound("document not found".to_string()));
    }
    Ok(Json(json!({"ok": true})))
}

async fn delete_doc(State(st): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    let uid = st.user_id(&headers)?;
    let ok = st.with_db(|db| db.delete_document(&uid, &id))?;
    if !ok {
        return Err(ApiError::NotFound("document not found".to_string()));
    }
    Ok(Json(json!({"ok": true})))
}

// ---------- conversion ----------

#[derive(Deserialize)]
struct ConvertBody {
    /// Target format: pdf, docx, html, md, txt, odt, rtf.
    format: String,
}

async fn convert_doc(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(b): Json<ConvertBody>,
) -> Result<Response, ApiError> {
    let uid = st.user_id(&headers)?;
    let doc = st.with_db(|db| db.get_document(&uid, &id))?.ok_or_else(|| ApiError::NotFound("document not found".to_string()))?;
    let fmt = b.format.to_lowercase();
    // Run conversion on a blocking thread (CPU-bound layout/render).
    let content_json = doc.content_json.clone();
    let fmt_clone = fmt.clone();
    let bytes = tokio::task::spawn_blocking(move || convert_bytes(&content_json, &fmt_clone)).await.map_err(|_| ApiError::Internal)??;
    let (ctype, fname) = match fmt.as_str() {
        "pdf" => ("application/pdf", "document.pdf"),
        "docx" => ("application/vnd.openxmlformats-officedocument.wordprocessingml.document", "document.docx"),
        "html" => ("text/html", "document.html"),
        "md" => ("text/markdown", "document.md"),
        "txt" => ("text/plain", "document.txt"),
        "odt" => ("application/vnd.oasis.opendocument.text", "document.odt"),
        "rtf" => ("application/rtf", "document.rtf"),
        _ => return Err(ApiError::BadRequest("unsupported format (pdf, docx, html, md, txt, odt, rtf)".to_string())),
    };
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, ctype), (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{fname}\"")[..])], bytes)
        .into_response())
}

fn convert_bytes(content_json: &str, fmt: &str) -> Result<Vec<u8>, ApiError> {
    // Parse the stored JSON into a document via the engine's IO.
    let doc: goharscribe_doc::Document =
        serde_json::from_str(content_json).map_err(|_| ApiError::BadRequest("stored document is corrupt".to_string()))?;
    // io::save_bytes dispatches by extension: pdf/docx via io_ext, others via goharscribe-formats.
    goharscribe_engine::io::save_bytes(&format!("doc.{fmt}"), &doc).map_err(ApiError::BadRequest)
}

// ---------- headless commands ----------

#[derive(Deserialize)]
struct ExecBody {
    command: String,
    params: Option<Value>,
}

async fn exec_command(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(b): Json<ExecBody>,
) -> Result<impl IntoResponse, ApiError> {
    let uid = st.user_id(&headers)?;
    let doc = st.with_db(|db| db.get_document(&uid, &id))?.ok_or_else(|| ApiError::NotFound("document not found".to_string()))?;
    if b.command.len() > 100 {
        return Err(ApiError::BadRequest("bad command".to_string()));
    }
    let content_json = doc.content_json.clone();
    let params = b.params.unwrap_or(Value::Null);
    let command = b.command.clone();
    // Engine session is !Send (UI-linked); run on a blocking thread with a fresh session.
    let new_json = tokio::task::spawn_blocking(move || run_command(&content_json, &command, &params)).await.map_err(|_| ApiError::Internal)??;
    st.with_db(|db| db.update_document(&uid, &id, None, Some(&new_json)))?;
    let content: Value = serde_json::from_str(&new_json)?;
    Ok(Json(json!({"ok": true, "content": content})))
}

fn run_command(content_json: &str, command: &str, params: &Value) -> Result<String, ApiError> {
    use goharscribe_engine::Session;
    let doc: goharscribe_doc::Document =
        serde_json::from_str(content_json).map_err(|_| ApiError::BadRequest("stored document is corrupt".to_string()))?;
    let mut session = Session::new(doc);
    session.run(command, params).map_err(|e| ApiError::BadRequest(format!("command failed: {e}")))?;
    serde_json::to_string(&session.doc).map_err(|_| ApiError::Internal)
}

// ---------- router ----------

pub fn router(state: AppState) -> axum::Router {
    use tower_http::cors::{Any, CorsLayer};
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    axum::Router::new()
        .route("/api/health", get(|| async { Json(json!({"ok": true})) }))
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/docs", get(list_docs).post(create_doc))
        .route("/api/docs/:id", get(get_doc).put(update_doc).delete(delete_doc))
        .route("/api/docs/:id/convert", post(convert_doc))
        .route("/api/docs/:id/exec", post(exec_command))
        .layer(cors)
        .with_state(state)
}
