use actix_web::{post, web::Json, HttpResponse};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::Deserialize;
use std::env;
use once_cell::sync::Lazy;
use chrono::Utc;

static API_SECRET: Lazy<String> = Lazy::new(|| env::var("CLOUDINARY_API_SECRET").unwrap());
static API_KEY: Lazy<String>    = Lazy::new(|| env::var("CLOUDINARY_API_KEY").unwrap());
static CLOUD_NAME: Lazy<String>= Lazy::new(|| env::var("CLOUDINARY_CLOUD_NAME").unwrap());

#[derive(Deserialize)]
pub struct SignRequest {
    pub folder: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SignResponse {
    signature: String,
    timestamp: i64,
    api_key: String,
    upload_url: String,
}

#[post("/sign")]
pub async fn sign_upload(req: Json<SignRequest>) -> HttpResponse {
    let timestamp = Utc::now().timestamp();
    // /auto/upload? timestamp & folder if present
    let mut to_sign = format!("timestamp={}", timestamp);
    if let Some(f) = &req.folder {
        to_sign.push_str(&format!("&folder={}", f));
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(API_SECRET.as_bytes()).unwrap();
    mac.update(to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    HttpResponse::Ok().json(SignResponse {
        signature,
        timestamp,
        api_key: API_KEY.clone(),
        upload_url: format!("https://api.cloudinary.com/v1_1/{}/auto/upload", CLOUD_NAME.clone()),
    })
}
