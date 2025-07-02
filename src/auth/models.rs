use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime; // Changed from chrono::{DateTime, Utc}
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: OffsetDateTime, // Changed to OffsetDateTime
    pub updated_at: OffsetDateTime, // Changed to OffsetDateTime
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub role: String,
    pub exp: usize, // This remains usize as typically expected by jsonwebtoken
}

// Make Claims an Actix extractor
use actix_web::{FromRequest, dev::Payload, HttpRequest, HttpMessage, Error as ActixError};
use futures_util::future::{ready, Ready};

impl FromRequest for Claims {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(c) = req.extensions().get::<Claims>().cloned() {
            ready(Ok(c))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Missing or invalid JWT")))
        }
    }
}