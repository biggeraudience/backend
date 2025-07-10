use axum::http::StatusCode;
use axum::response::IntoResponse;

// Liveness probe
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
