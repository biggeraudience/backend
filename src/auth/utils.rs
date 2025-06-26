use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::auth::models::Claims;
use crate::error::AppError;
use chrono::Utc;
use std::env;

pub async fn hash_password(password: &str) -> Result<String, AppError> {
    Ok(hash(password, DEFAULT_COST)?)
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    Ok(verify(password, hash)?)
}

pub fn create_jwt(claims: Claims, secret: &str) -> Result<String, AppError> {
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}
