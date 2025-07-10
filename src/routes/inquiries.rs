use axum::{Router, routing::{get, post, put, delete}, extract::{Extension, Path, Json}, response::IntoResponse, http::StatusCode};
use serde::{Serialize, Deserialize};
use crate::db::DbPool;
use uuid::Uuid;

#[derive(Serialize)]
struct Inquiry { id: Uuid, name: String, email: String, subject: String, message: String, status: String, response: Option<String>, created_at: chrono::DateTime<chrono::Utc> }
#[derive(Deserialize)]
struct NewInquiry { name: String, email: String, subject: String, message: String }
#[derive(Deserialize)]
struct StatusUpdate { status: String, response: Option<String> }

// Public endpoint
async fn create_inquiry(Json(payload): Json<NewInquiry>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query!("INSERT INTO inquiries(id, name, email, subject, message, status, created_at) VALUES($1,$2,$3,$4,$5,'New',$6)",
        id, payload.name, payload.email, payload.subject, payload.message, now)
        .execute(&*pool).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    (StatusCode::CREATED, Json(id))
}

// Admin endpoints
async fn list_inquiries(Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    let recs = sqlx::query_as!(Inquiry, "SELECT id, name, email, subject, message, status, response, created_at FROM inquiries ORDER BY created_at DESC")
        .fetch_all(&*pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    (StatusCode::OK, Json(recs))
}

async fn update_status(Path(id): Path<Uuid>, Json(payload): Json<StatusUpdate>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    sqlx::query!("UPDATE inquiries SET status=$1, response=$2 WHERE id=$3", payload.status, payload.response, id)
        .execute(&*pool).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    StatusCode::NO_CONTENT
}

async fn delete_inquiry(Path(id): Path<Uuid>, Extension(pool): Extension<DbPool>) -> impl IntoResponse {
    sqlx::query!("DELETE FROM inquiries WHERE id=$1", id)
        .execute(&*pool).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    StatusCode::NO_CONTENT
}

pub fn router() -> Router {
    Router::new()
        .route("/inquiries", post(create_inquiry).get(list_inquiries))
        .route("/inquiries/:id/status", put(update_status))
        .route("/inquiries/:id", delete(delete_inquiry))
}
