use actix_web::{get, post, put, delete, web, HttpResponse};
use sqlx::PgPool;
use web::{Data, Json};
use uuid::Uuid;
use chrono::Utc;

use crate::auth::models::Claims; // To get user ID if authenticated
use crate::error::AppError;
use crate::inquiries::models::{Inquiry, CreateInquiryPayload, UpdateInquiryStatusPayload};

// Public Endpoint
#[post("/")]
pub async fn submit_inquiry(
    pool: Data<PgPool>,
    claims: Option<Claims>, // Option because inquiry can be from unregistered user
    payload: Json<CreateInquiryPayload>,
) -> Result<HttpResponse, AppError> {
    // Basic validation
    if payload.name.is_empty() || payload.email.is_empty() || payload.message.is_empty() {
        return Err(AppError::ValidationError("Name, email, and message are required.".to_string()));
    }

    let user_id = claims.map(|c| c.user_id);

    let new_inquiry = sqlx::query_as!(
        Inquiry,
        r#"
        INSERT INTO inquiries (user_id, name, email, phone, subject, message, status)
        VALUES (, , , , , , 'new')
        RETURNING id, user_id, name, email, phone, subject, message, status, created_at, updated_at
        "#,
        user_id,
        payload.name,
        payload.email,
        payload.phone,
        payload.subject,
        payload.message
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(new_inquiry))
}

// Admin Endpoints
#[get("/")]
pub async fn list_inquiries(pool: Data<PgPool>) -> Result<HttpResponse, AppError> {
    let inquiries = sqlx::query_as!(
        Inquiry,
        r#"
        SELECT id, user_id, name, email, phone, subject, message, status, created_at, updated_at
        FROM inquiries
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(inquiries))
}

#[put("/{inquiry_id}/status")]
pub async fn update_inquiry_status(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
    payload: Json<UpdateInquiryStatusPayload>,
) -> Result<HttpResponse, AppError> {
    let inquiry_id = path.into_inner();
    let new_status = payload.status.clone();

    // Basic status validation
    if !["new", "in_progress", "resolved", "closed"].contains(&new_status.as_str()) {
        return Err(AppError::ValidationError("Invalid status. Must be 'new', 'in_progress', 'resolved', or 'closed'".to_string()));
    }

    let updated_inquiry = sqlx::query_as!(
        Inquiry,
        r#"
        UPDATE inquiries
        SET status = 
        WHERE id = 
        RETURNING id, user_id, name, email, phone, subject, message, status, created_at, updated_at
        "#,
        new_status,
        inquiry_id
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(updated_inquiry))
}

#[delete("/{inquiry_id}")]
pub async fn delete_inquiry(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let inquiry_id = path.into_inner();

    let deleted = sqlx::query!(
        r#"
        DELETE FROM inquiries
        WHERE id = 
        RETURNING id
        "#,
        inquiry_id
    )
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if deleted == 0 {
        return Err(AppError::NotFound("Inquiry".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}
