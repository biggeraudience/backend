use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation}; // Added decode
use crate::auth::models::Claims;
use crate::error::AppError;
use time::OffsetDateTime; // Keep OffsetDateTime as it's relevant for JWT expiration

pub async fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST).map_err(AppError::BcryptError)
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash).map_err(AppError::BcryptError)
}

pub fn create_jwt(claims: Claims, secret: &str) -> Result<String, AppError> {
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(AppError::JwtError)?;
    Ok(token)
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(AppError::JwtError)
}