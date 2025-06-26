use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, FromRequest,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use web::Data;
use actix_web::web;

use crate::auth::models::Claims;
use crate::auth::utils::decode_jwt;
use crate::error::AppError;

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: actix_web::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware { service })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: actix_web::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_secret_data: Option<Data<String>> = req.app_data::<Data<String>>().cloned();
        let jwt_secret = match jwt_secret_data {
            Some(data) => data.as_ref().clone(),
            None => {
                let err = AppError::InternalServerError.into();
                return Box::pin(async { Err(err) });
            }
        };

        let auth_header = req.headers().get("Authorization");

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = if let Some(header_value) = auth_header {
                if let Ok(header_str) = header_value.to_str() {
                    if header_str.starts_with("Bearer ") {
                        let token = &header_str[7..]; // Strip "Bearer "
                        match decode_jwt(token, &jwt_secret) {
                            Ok(claims) => {
                                let mut req = fut.await?.into_request(); // Get mutable request from future
                                req.extensions_mut().insert(claims); // Insert claims into request extensions
                                ServiceResponse::new(req, fut.await?.into_response()) // Reconstruct ServiceResponse
                            },
                            Err(e) => {
                                tracing::warn!("JWT decoding failed: {}", e);
                                return Err(AppError::Unauthorized.into());
                            }
                        }
                    } else {
                        return Err(AppError::Unauthorized.into());
                    }
                } else {
                    return Err(AppError::Unauthorized.into());
                }
            } else {
                return Err(AppError::Unauthorized.into());
            };
            Ok(res)
        })
    }
}

// Custom Extractor for Claims, used by handlers
impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(claims) = req.extensions().get::<Claims>() {
            ok(claims.clone())
        } else {
            // This case should ideally not happen if JwtAuth middleware is always applied
            // before routes that require Claims.
            // If it does, it means the token was missing or invalid, and JwtAuth should have
            // already returned Unauthorized. This is a fallback.
            Ready::Err(AppError::Unauthorized.into())
        }
    }
}

pub struct AdminRoleCheck;

impl<S, B> Transform<S, ServiceRequest> for AdminRoleCheck
where
    S: actix_web::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminRoleCheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminRoleCheckMiddleware { service })
    }
}

pub struct AdminRoleCheckMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::Service<ServiceRequest> for AdminRoleCheckMiddleware<S>
where
    S: actix_web::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let claims = req.extensions().get::<Claims>().cloned(); // Get claims from extensions

        let fut = self.service.call(req); // Call the next service in the chain

        Box::pin(async move {
            match claims {
                Some(claims) if claims.role == "admin" => {
                    fut.await // If admin, proceed
                }
                _ => {
                    tracing::warn!("Non-admin attempt to access admin route. User ID: {:?}", claims.map(|c| c.user_id));
                    Err(AppError::Forbidden.into()) // If not admin or no claims, return forbidden
                }
            }
        })
    }
}
