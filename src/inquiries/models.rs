use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inquiry {
    pub id: Uuid,
    pub user_id: Option<Uuid>, // Matches `ON DELETE SET NULL` in DB
    pub name: String,
    pub email: String,
    pub phone: Option<String>, // Matches nullable `phone TEXT` in DB
    pub subject: Option<String>, // Added: Matches nullable `subject TEXT` in DB
    pub message: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateInquiryPayload {
    // user_id is now Option<Uuid> as per the Inquiry struct
    pub user_id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub subject: Option<String>, // Added
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInquiryStatusPayload {
    pub status: String,
}