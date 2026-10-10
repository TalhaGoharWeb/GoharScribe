//! GoharScribe SaaS backend: multi-user document API with authentication.
//!
//! Layer 5 (headless service, like `mcp`). Provides:
//! - User signup/login with Argon2 password hashing and JWT tokens
//! - Per-user document CRUD (JSON document format)
//! - Document conversion (PDF, DOCX, HTML, Markdown, TXT)
//! - Headless command execution via the engine
//!
//! Never panics: all errors are `Result<T, ApiError>`.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

pub mod api;
pub mod auth;
pub mod db;
pub mod error;

pub use api::{AppState, router};
pub use db::Db;
pub use error::ApiError;
