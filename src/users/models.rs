// src/users/models.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// A user record.  
/// Note: we derive `Deserialize` as well, which can be handy for testing or logging.
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

/// Payload for a user to update their own profile.
#[derive(Debug, Deserialize)]
pub struct UpdateProfilePayload {
    pub username: Option<String>,
    pub email: Option<String>,
    // Add other optional profile fields here as needed
}

/// Payload for an admin to update another user's role.
#[derive(Debug, Deserialize)]
pub struct UpdateUserRolePayload {
    pub role: String,
}
