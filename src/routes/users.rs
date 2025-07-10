use axum::{
    Router,
    routing::{get, put},
    extract::{Extension, Path, Json, TypedHeader},
    response::IntoResponse,
    http::StatusCode,
    headers::{Authorization, authorization::Bearer},
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::db::DbPool;
use crate::utils::jwt;

#[derive(Serialize)]
struct Me {
    id: Uuid,
    email: String,
    role: String,
}

#[derive(Deserialize)]
struct UpdateMe {
    email: Option<String>,
    password: Option<String>, // Handle password update logic later
}

#[derive(Serialize)]
struct UserSummary {
    id: Uuid,
    email: String,
    role: String,
    status: String,
}

#[derive(Deserialize)]
struct RoleUpdate {
    role: String,
}

async fn get_me(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Extension(pool): Extension<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let claims = jwt::verify_token(bearer.token())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid subject"))?;

    let rec = sqlx::query_as!(Me, "SELECT id, email, role FROM users WHERE id=$1", id)
        .fetch_one(&*pool)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "User not found"))?;

    Ok((StatusCode::OK, Json(rec)))
}

async fn update_me(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<UpdateMe>,
    Extension(pool): Extension<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let claims = jwt::verify_token(bearer.token())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid subject"))?;

    if let Some(email) = payload.email {
        sqlx::query!("UPDATE users SET email = $1 WHERE id = $2", email, id)
            .execute(&*pool)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Failed to update email"))?;
    }

    // TODO: Handle password update logic

    Ok(StatusCode::NO_CONTENT)
}

async fn list_users(
    Extension(pool): Extension<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let users = sqlx::query_as!(
        UserSummary,
        "SELECT id, email, role, status FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users"))?;

    Ok((StatusCode::OK, Json(users)))
}

async fn update_role(
    Path(id): Path<Uuid>,
    Json(payload): Json<RoleUpdate>,
    Extension(pool): Extension<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    sqlx::query!("UPDATE users SET role = $1 WHERE id = $2", payload.role, id)
        .execute(&*pool)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Failed to update role"))?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router {
    Router::new()
        .route("/users/me", get(get_me).put(update_me))
        .route("/users", get(list_users))
        .route("/users/:id/role", put(update_role))
}
