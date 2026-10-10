//! SQLite storage: users and documents. All queries are parameterized.
//! Runs on a dedicated blocking thread via `tokio::task::spawn_blocking`.

use chrono::Utc;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::error::ApiError;

pub struct Db {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub content_json: String,
    pub updated_at: String,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, ApiError> {
        let conn = Connection::open(path).map_err(|_| ApiError::Internal)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_memory() -> Result<Self, ApiError> {
        let conn = Connection::open_in_memory().map_err(|_| ApiError::Internal)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), ApiError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    email TEXT UNIQUE NOT NULL,
                    password_hash TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS documents (
                    id TEXT PRIMARY KEY,
                    owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                    title TEXT NOT NULL,
                    content_json TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_docs_owner ON documents(owner_id);",
            )
            .map_err(|_| ApiError::Internal)?;
        Ok(())
    }

    pub fn create_user(&self, email: &str, password_hash: &str) -> Result<User, ApiError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        match self
            .conn
            .execute("INSERT INTO users (id, email, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)", params![id, email, password_hash, now])
        {
            Ok(_) => Ok(User { id, email: email.to_string(), password_hash: password_hash.to_string(), created_at: now }),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("UNIQUE constraint failed") {
                    Err(ApiError::Conflict("email already registered".to_string()))
                } else {
                    Err(ApiError::Internal)
                }
            }
        }
    }

    pub fn find_user_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let mut stmt =
            self.conn.prepare("SELECT id, email, password_hash, created_at FROM users WHERE email = ?1").map_err(|_| ApiError::Internal)?;
        let mut rows = stmt.query(params![email]).map_err(|_| ApiError::Internal)?;
        match rows.next().map_err(|_| ApiError::Internal)? {
            Some(r) => Ok(Some(User {
                id: r.get(0).map_err(|_| ApiError::Internal)?,
                email: r.get(1).map_err(|_| ApiError::Internal)?,
                password_hash: r.get(2).map_err(|_| ApiError::Internal)?,
                created_at: r.get(3).map_err(|_| ApiError::Internal)?,
            })),
            None => Ok(None),
        }
    }

    pub fn create_document(&self, owner_id: &str, title: &str, content_json: &str) -> Result<Document, ApiError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO documents (id, owner_id, title, content_json, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, owner_id, title, content_json, now],
            )
            .map_err(|_| ApiError::Internal)?;
        Ok(Document { id, owner_id: owner_id.to_string(), title: title.to_string(), content_json: content_json.to_string(), updated_at: now })
    }

    pub fn list_documents(&self, owner_id: &str) -> Result<Vec<Document>, ApiError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, owner_id, title, content_json, updated_at FROM documents WHERE owner_id = ?1 ORDER BY updated_at DESC")
            .map_err(|_| ApiError::Internal)?;
        let rows = stmt
            .query_map(params![owner_id], |r| {
                Ok(Document { id: r.get(0)?, owner_id: r.get(1)?, title: r.get(2)?, content_json: r.get(3)?, updated_at: r.get(4)? })
            })
            .map_err(|_| ApiError::Internal)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|_| ApiError::Internal)?);
        }
        Ok(out)
    }

    pub fn get_document(&self, owner_id: &str, doc_id: &str) -> Result<Option<Document>, ApiError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, owner_id, title, content_json, updated_at FROM documents WHERE id = ?1 AND owner_id = ?2")
            .map_err(|_| ApiError::Internal)?;
        let mut rows = stmt.query(params![doc_id, owner_id]).map_err(|_| ApiError::Internal)?;
        match rows.next().map_err(|_| ApiError::Internal)? {
            Some(r) => Ok(Some(Document {
                id: r.get(0).map_err(|_| ApiError::Internal)?,
                owner_id: r.get(1).map_err(|_| ApiError::Internal)?,
                title: r.get(2).map_err(|_| ApiError::Internal)?,
                content_json: r.get(3).map_err(|_| ApiError::Internal)?,
                updated_at: r.get(4).map_err(|_| ApiError::Internal)?,
            })),
            None => Ok(None),
        }
    }

    pub fn update_document(&self, owner_id: &str, doc_id: &str, title: Option<&str>, content_json: Option<&str>) -> Result<bool, ApiError> {
        let now = Utc::now().to_rfc3339();
        // Fetch current values for fields not being updated.
        let current = match self.get_document(owner_id, doc_id)? {
            Some(d) => d,
            None => return Ok(false),
        };
        let title = title.unwrap_or(&current.title);
        let content_json = content_json.unwrap_or(&current.content_json);
        let n = self
            .conn
            .execute(
                "UPDATE documents SET title = ?1, content_json = ?2, updated_at = ?3 WHERE id = ?4 AND owner_id = ?5",
                params![title, content_json, now, doc_id, owner_id],
            )
            .map_err(|_| ApiError::Internal)?;
        Ok(n > 0)
    }

    pub fn delete_document(&self, owner_id: &str, doc_id: &str) -> Result<bool, ApiError> {
        let n =
            self.conn.execute("DELETE FROM documents WHERE id = ?1 AND owner_id = ?2", params![doc_id, owner_id]).map_err(|_| ApiError::Internal)?;
        Ok(n > 0)
    }
}
