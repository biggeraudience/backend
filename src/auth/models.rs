// src/auth/models.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

// ───────────────────────────────────────────────────────────────────────────
// ─── User Model ─────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// ───────────────────────────────────────────────────────────────────────────
// ─── Auth Payloads & Responses ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthTokenResponse {
    pub token: String,
    pub user_id: Uuid,
    pub role: String,
}

// ───────────────────────────────────────────────────────────────────────────
// ─── JWT Claims & Actix Extractor ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub role: String,
    pub exp: usize,
}

use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    FromRequest,
    HttpMessage,
    HttpRequest,
};
use futures_util::future::{ready, Ready};

impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Look for our decoded Claims in request extensions
        if let Some(claims) = req.extensions().get::<Claims>().cloned() {
            ready(Ok(claims))
        } else {
            ready(Err(ErrorUnauthorized("Missing or invalid JWT")))
        }
    }
}
