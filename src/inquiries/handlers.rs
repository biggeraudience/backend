use actix_web::{web, HttpResponse, post, get, put, delete};
use sqlx::{PgPool, FromRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

use crate::error::AppError;

// --- Struct Definitions ---
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inquiry {
    pub id: Uuid,
    pub user_id: Option<Uuid>, // Matches `ON DELETE SET NULL` in DB
    pub name: String,
    pub email: String,
    pub phone: Option<String>, // Matches nullable `phone TEXT` in DB
    pub subject: Option<String>, // Matches nullable `subject TEXT` in DB
    pub message: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateInquiryPayload {
    pub user_id: Option<Uuid>,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub subject: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInquiryStatusPayload {
    pub status: String,
}

// --- Handler Functions ---

/// Handles creating a new inquiry.
#[post("/inquiries")]
pub async fn create_inquiry(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateInquiryPayload>,
) -> Result<HttpResponse, AppError> {
    let now = OffsetDateTime::now_utc();

    let inquiry = sqlx::query_as!(
        Inquiry,
        r#"
        INSERT INTO inquiries (user_id, name, email, phone, subject, message, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, name, email, phone, subject, message, status, created_at, updated_at
        "#,
        payload.user_id,
        payload.name,
        payload.email,
        payload.phone,
        payload.subject,
        payload.message,
        "pending", // Default status for new inquiries
        now,
        now
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(inquiry))
}

/// Handles fetching all inquiries.
#[get("/inquiries")]
pub async fn get_all_inquiries(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let items = sqlx::query_as!(
        Inquiry,
        r#"SELECT id, user_id, name, email, phone, subject, message, status, created_at, updated_at FROM inquiries ORDER BY created_at DESC"#
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(items))
}

/// Handles fetching a single inquiry by ID.
#[get("/inquiries/{id}")]
pub async fn get_inquiry_detail(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let inquiry = sqlx::query_as!(
        Inquiry,
        r#"SELECT id, user_id, name, email, phone, subject, message, status, created_at, updated_at FROM inquiries WHERE id = $1"#,
        id
    )
    .fetch_optional(&**pool)
    .await?;

    match inquiry {
        Some(i) => Ok(HttpResponse::Ok().json(i)),
        None => Err(AppError::NotFound(format!("Inquiry with id {} not found", id))),
    }
}

/// Handles updating the status of an inquiry.
#[put("/inquiries/{id}/status")]
pub async fn update_inquiry_status(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    payload: web::Json<UpdateInquiryStatusPayload>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();
    let now = OffsetDateTime::now_utc();

    let inquiry = sqlx::query_as!(
        Inquiry,
        r#"
        UPDATE inquiries SET status = $1, updated_at = $2
        WHERE id = $3
        RETURNING id, user_id, name, email, phone, subject, message, status, created_at, updated_at
        "#,
        payload.status,
        now,
        id
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(inquiry))
}

/// Handles deleting an inquiry.
#[delete("/inquiries/{id}")]
pub async fn delete_inquiry(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let id = path.into_inner();

    let deleted_rows = sqlx::query!("DELETE FROM inquiries WHERE id = $1", id)
        .execute(&**pool)
        .await?
        .rows_affected();

    if deleted_rows == 0 {
        return Err(AppError::NotFound(format!("Inquiry with id {} not found", id)));
    }

    Ok(HttpResponse::NoContent().finish())
}