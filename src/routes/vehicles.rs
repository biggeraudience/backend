use axum::{
  Router, routing::{get, post, put, delete},
  extract::{Extension, Path, Json},
  response::IntoResponse, http::StatusCode,
};
use serde::{Serialize, Deserialize};
use crate::{db::DbPool, utils::auth::{AuthUser, AdminUser}};
use uuid::Uuid;
use reqwest::Client;

#[derive(Serialize)]
struct Vehicle { /* same as before */ }
#[derive(Deserialize)]
struct VehiclePayload { /* same as before */ }

async fn list_vehicles(_: AuthUser, Extension(pool): Extension<DbPool>) -> impl IntoResponse { /* ... */ }

async fn get_vehicle(_: AuthUser, Path(id): Path<Uuid>, Extension(pool): Extension<DbPool>) -> impl IntoResponse { /* ... */ }

async fn create_vehicle(AdminUser(_), Json(payload): Json<VehiclePayload>, Extension(pool): Extension<DbPool>) -> impl IntoResponse { /* ... */ }

async fn update_vehicle(AdminUser(_), Path(id): Path<Uuid>, Json(payload): Json<VehiclePayload>, Extension(pool): Extension<DbPool>) -> impl IntoResponse { /* ... */ }

async fn delete_vehicle(AdminUser(_), Path(id): Path<Uuid>, Extension(pool): Extension<DbPool>) -> impl IntoResponse { /* ... */ }

// Cloudinary upload
async fn upload_image(
  AdminUser(_),
  TypedHeader(content_type): TypedHeader<headers::ContentType>,
  bytes: bytes::Bytes,
) -> impl IntoResponse {
  let cfg = crate::config::get();
  let form = reqwest::multipart::Form::new()
    .text("upload_preset", cfg.cloudinary_upload_preset.clone())
    .part("file", reqwest::multipart::Part::bytes(bytes).mime_str(content_type.as_ref()).unwrap());
  let client = Client::new();
  let res = client.post(format!(
      "https://api.cloudinary.com/v1_1/{}/image/upload",
      cfg.cloudinary_cloud_name
    ))
    .multipart(form)
    .basic_auth(&cfg.cloudinary_api_key, Some(&cfg.cloudinary_api_secret))
    .send().await.map_err(|_| StatusCode::BAD_REQUEST)?;
  let body = res.json::<serde_json::Value>().await.map_err(|_| StatusCode::BAD_REQUEST)?;
  (StatusCode::OK, Json(body))
}

pub fn router() -> Router {
  Router::new()
    .route("/vehicles", get(list_vehicles).post(create_vehicle))
    .route("/vehicles/upload", post(upload_image))
    .route("/vehicles/:id", get(get_vehicle).put(update_vehicle).delete(delete_vehicle))
}
