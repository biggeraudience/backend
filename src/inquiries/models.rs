/// src/inquiries/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inquiry {
    pub id:         Uuid,
    pub user_id:    Option<Uuid>,    // UUID NULLABLE
    pub name:       String,          // TEXT NOT NULL
    pub email:      String,          // TEXT NOT NULL
    pub phone:      Option<String>,  // TEXT NULLABLE
    pub subject:    Option<String>,  // TEXT NULLABLE
    pub message:    String,          // TEXT NOT NULL
    pub status:     String,          // TEXT NOT NULL
    pub created_at: OffsetDateTime,  // TIMESTAMPTZ NOT NULL
    pub updated_at: OffsetDateTime,  // TIMESTAMPTZ NOT NULL
}
