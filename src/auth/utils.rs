use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use chrono::Utc;
use std::env;
use crate::error::AppError;
use crate::auth::models::Claims;

pub async fn hash_password(password: &str) -> Result<String, AppError> {
    Ok(hash(password, DEFAULT_COST).map_err(|e| AppError::AuthError(e.to_string()))?)
}

pub async fn verify_password(password: &str, hashed_password: &str) -> Result<bool, AppError> {
    Ok(verify(password, hashed_password).map_err(|e| AppError::AuthError(e.to_string()))?)
}

pub fn create_jwt(claims: Claims, jwt_secret: &str) -> Result<String, AppError> {
    let token = encode(
        &Header::new(Algorithm::HS512),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn decode_jwt(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS512);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )?;
    Ok(token_data.claims)
}
