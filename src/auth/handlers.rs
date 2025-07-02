use actix_web::{post, web, HttpResponse};
use actix_web::web::Data;
use sqlx::PgPool;
use time::{OffsetDateTime, Duration};

use crate::auth::models::{RegisterPayload, LoginPayload, AuthTokenResponse, User, Claims};
use crate::auth::utils::{hash_password, verify_password, create_jwt};
use crate::error::AppError;

#[post("/register")]
pub async fn register_user(
    pool: Data<PgPool>,
    payload: web::Json<RegisterPayload>,
) -> Result<HttpResponse, AppError> {
    if payload.username.is_empty() || payload.email.is_empty() || payload.password.len() < 8 {
        return Err(AppError::ValidationError(
            "Username, email, and password (min 8 chars) are required.".into(),
        ));
    }

    // Hash the password
    let hashed = hash_password(&payload.password).await?;
    let now = OffsetDateTime::now_utc();

    // Insert & return the new user
    let new_user: User = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, role, created_at, updated_at)
        VALUES ($1, $2, $3, 'user', $4, $4)
        RETURNING id, username, email, password_hash, role, created_at, updated_at
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hashed)
    .bind(now)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(new_user))
}

#[post("/login")]
pub async fn login_user(
    pool: Data<PgPool>,
    jwt_secret: Data<String>,
    payload: web::Json<LoginPayload>,
) -> Result<HttpResponse, AppError> {
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::ValidationError("Email and password are required.".into()));
    }

    // Fetch user by email
    let user: User = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, role, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&payload.email)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::AuthError("Invalid credentials.".into()))?;

    // Check password
    if !verify_password(&payload.password, &user.password_hash).await? {
        return Err(AppError::AuthError("Invalid credentials.".into()));
    }

    // Create JWT
    let exp = (OffsetDateTime::now_utc() + Duration::days(7)).unix_timestamp() as usize;
    let claims = Claims { user_id: user.id, role: user.role.clone(), exp };
    let token = create_jwt(claims, jwt_secret.get_ref())?;

    Ok(HttpResponse::Ok().json(AuthTokenResponse {
        token,
        user_id: user.id,
        role: user.role,
    }))
}
