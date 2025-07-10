use axum::{Router, routing::post, extract::Extension, Json, response::IntoResponse, http::StatusCode};
use serde::Deserialize;
use sqlx::types::Uuid;
use crate::db::DbPool;
use crate::utils::jwt::{self, create_token, JwtError};
use bcrypt::{hash, verify};

#[derive(Deserialize)] struct Login { email: String, password: String }

async fn login(Json(payload): Json<Login>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let rec = sqlx::query!("SELECT id, password_hash, role FROM users WHERE email=$1", payload.email)
        .fetch_one(&*pool).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if verify(&payload.password, &rec.password_hash).unwrap() {
       let token = create_token(&rec.id.to_string(), &rec.role);
        (StatusCode::OK, Json(serde_json::json!({"token": token, "user_id": rec.id, "role": rec.role})))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn register(Json(payload): Json<Login>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let hash = hash(&payload.password, 12).unwrap();
    let id = Uuid::new_v4();
    sqlx::query!("INSERT INTO users(id,email,password_hash,role) VALUES($1,$2,$3,'user')", id, payload.email, hash)
        .execute(&*pool).await.map_err(|_| StatusCode::CONFLICT)?;
    (StatusCode::CREATED, Json(serde_json::json!({"user_id": id})))
}

pub fn router() -> Router { Router::new().route("/auth/login", post(login)).route("/auth/register", post(register)) }
