use actix_web::{post, web, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;
use crate::auth::models::{RegisterPayload, LoginPayload, AuthTokenResponse, User, Claims};
use crate::auth::utils::{hash_password, verify_password, create_jwt};
use crate::error::AppError;
use actix_web::web::Data;

#[post("/register")]
pub async fn register(
    pool: Data<PgPool>,
    payload: web::Json<RegisterPayload>,
) -> Result<HttpResponse, AppError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.len() < 8 {
        return Err(AppError::ValidationError(
            "Username, email, and password (min 8 chars) are required.".to_string(),
        ));
    }

    let hashed = hash_password(&payload.password).await?;
    let new_user: User = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, 'user')
        RETURNING id, username, email, password_hash, role, created_at, updated_at
        "#,
        payload.username,
        payload.email,
        hashed
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(new_user))
}

#[post("/login")]
pub async fn login(
    pool: Data<PgPool>,
    jwt_secret: Data<String>,
    payload: web::Json<LoginPayload>,
) -> Result<HttpResponse, AppError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::ValidationError("Email and password are required.".to_string()));
    }

    let user: User = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash, role, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
        payload.email
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::AuthError("Invalid credentials.".to_string()))?;

    if !verify_password(&payload.password, &user.password_hash).await? {
        return Err(AppError::AuthError("Invalid credentials.".to_string()));
    }

    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims { user_id: user.id, role: user.role.clone(), exp };
    let token = create_jwt(claims, jwt_secret.get_ref())?;

    Ok(HttpResponse::Ok().json(AuthTokenResponse {
        token,
        user_id: user.id,
        role: user.role,
    }))
}
