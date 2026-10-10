//! Authentication: Argon2 password hashing and JWT bearer tokens.

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    email: String,
    exp: usize,
}

#[derive(Clone)]
pub struct Auth {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Auth {
    /// `secret` must be at least 32 bytes of randomness in production.
    /// Pass via `GOHARSCRIBE_JWT_SECRET` env; never commit it.
    pub fn new(secret: &[u8]) -> Self {
        Self { encoding: EncodingKey::from_secret(secret), decoding: DecodingKey::from_secret(secret) }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default().hash_password(password.as_bytes(), &salt).map_err(|_| ApiError::Internal)?;
        Ok(hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, ApiError> {
        let parsed = PasswordHash::new(hash).map_err(|_| ApiError::Internal)?;
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
    }

    pub fn issue_token(&self, user_id: &str, email: &str) -> Result<String, ApiError> {
        let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
        let claims = Claims { sub: user_id.to_string(), email: email.to_string(), exp };
        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding).map_err(|_| ApiError::Internal)
    }

    pub fn verify_token(&self, token: &str) -> Result<(String, String), ApiError> {
        let data = jsonwebtoken::decode::<Claims>(token, &self.decoding, &Validation::default())
            .map_err(|_| ApiError::Unauthorized("invalid or expired token".to_string()))?;
        Ok((data.claims.sub, data.claims.email))
    }
}

/// Extract bearer token from `Authorization: Bearer <token>`.
pub fn bearer_token(headers: &axum::http::HeaderMap) -> Result<&str, ApiError> {
    let v = headers.get(axum::http::header::AUTHORIZATION).ok_or_else(|| ApiError::Unauthorized("missing Authorization header".to_string()))?;
    let s = v.to_str().map_err(|_| ApiError::Unauthorized("invalid Authorization header".to_string()))?;
    s.strip_prefix("Bearer ").ok_or_else(|| ApiError::Unauthorized("expected Bearer token".to_string()))
}
