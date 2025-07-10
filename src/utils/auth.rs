use axum::{
  async_trait,
  extract::{FromRequest, RequestParts},
  http::StatusCode,
  TypedHeader,
};
use axum::headers::{Authorization, authorization::Bearer};
use crate::utils::jwt;
use uuid::Uuid;

pub struct AuthUser {
  pub id: Uuid,
  pub role: String,
}

#[async_trait]
impl<B> FromRequest<B> for AuthUser
where
  B: Send,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    // extract Bearer token
    let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request(req)
      .await
      .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing token"))?;
let claims = jwt::verify_token(bearer.token())
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;
    let id = Uuid::parse_str(&claims.sub).map_err(|_| (StatusCode::UNAUTHORIZED, "Bad subject"))?;
    Ok(AuthUser { id, role: claims.role })
  }
}

pub struct AdminUser(pub AuthUser);

#[async_trait]
impl<B> FromRequest<B> for AdminUser
where
  B: Send,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
    let user = AuthUser::from_request(req).await?;
    if user.role == "admin" {
      Ok(AdminUser(user))
    } else {
      Err((StatusCode::FORBIDDEN, "Admin only"))
    }
  }
}
