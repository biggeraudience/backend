use actix_web::{get, put, web, HttpResponse};
use sqlx::PgPool;
use web::Data;
use uuid::Uuid;
use time::OffsetDateTime; // Import OffsetDateTime

use crate::auth::models::Claims;
use crate::error::AppError;
use crate::users::models::{User, UpdateProfilePayload, UpdateUserRolePayload};

#[get("/me")]
pub async fn get_me(
    claims: Claims,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let user: User = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, role, created_at, updated_at
        FROM users
        WHERE id = $1
        "#
    )
    .bind(claims.user_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(user))
}

#[put("/me")]
pub async fn update_me(
    claims: Claims,
    pool: Data<PgPool>,
    payload: web::Json<UpdateProfilePayload>,
) -> Result<HttpResponse, AppError> {
    let updated_user: User = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET username = COALESCE($1, username),
            email = COALESCE($2, email),
            updated_at = $3
        WHERE id = $4
        RETURNING id, username, email, password_hash, role, created_at, updated_at
        "#
    )
    .bind(payload.username.clone())
    .bind(payload.email.clone())
    .bind(OffsetDateTime::now_utc()) // Bind OffsetDateTime for updated_at
    .bind(claims.user_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(updated_user))
}

// Admin handlers
#[get("/")]
pub async fn list_users(
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let users: Vec<User> = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, role, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(users))
}

#[get("/{user_id}")]
pub async fn get_user_by_id(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let user: User = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, role, created_at, updated_at
        FROM users
        WHERE id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User".to_string()))?;

    Ok(HttpResponse::Ok().json(user))
}

#[put("/{user_id}/role")]
pub async fn update_user_role(
    path: web::Path<Uuid>,
    pool: Data<PgPool>,
    payload: web::Json<UpdateUserRolePayload>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let new_role = payload.role.clone();

    if !["user", "admin"].contains(&new_role.as_str()) {
        return Err(AppError::ValidationError("Invalid role specified. Must be 'user' or 'admin'".to_string()));
    }

    let updated_user: User = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET role = $1,
            updated_at = $2
        WHERE id = $3
        RETURNING id, username, email, password_hash, role, created_at, updated_at
        "#
    )
    .bind(new_role)
    .bind(OffsetDateTime::now_utc()) // Bind OffsetDateTime for updated_at
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(updated_user))
}