// src/users/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// Can reuse auth::models::User if no additional fields are needed,
// but separated for clarity and potential future divergence.
#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfilePayload {
    pub username: Option<String>,
    pub email: Option<String>,
    // Add other profile fields if needed
}

// For updating user role by admin
#[derive(Debug, Deserialize)]
pub struct UpdateUserRolePayload {
    pub role: String,
}