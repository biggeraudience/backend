// src/auth/middleware.rs
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready}, // Correct imports from dev
    Error,
    web,
};
use futures::future::{LocalBoxFuture, Ready, ok}; // Correct imports for futures
use std::{
    rc::Rc,
    task::{Context, Poll},
};

use crate::auth::models::Claims;
use crate::error::AppError;
use jsonwebtoken::{decode, DecodingKey, Validation};

// --- JwtAuthMiddleware ---

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware { service: Rc::new(service) })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Using the `forward_ready!` macro provided by actix_web::dev
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        // Skip JWT validation for specific paths
        let path = req.path();
        if path.starts_with("/auth/login") || path.starts_with("/auth/register") || path.starts_with("/inquiries") || path.starts_with("/vehicles") {
            return Box::pin(async move {
                svc.call(req).await
            });
        }

        Box::pin(async move {
            let req_claims = match req.headers().get("Authorization") {
                Some(header) => {
                    let auth_header = header.to_str().map_err(|_| AppError::Unauthorized)?;
                    if !auth_header.starts_with("Bearer ") {
                        return Err(AppError::Unauthorized.into());
                    }
                    let token_str = auth_header.trim_start_matches("Bearer ");

                    let jwt_secret = req
                        .app_data::<web::Data<String>>()
                        .expect("JWT Secret not configured");

                    let token_data = decode::<Claims>(
                        token_str,
                        &DecodingKey::from_secret(jwt_secret.as_bytes()),
                        &Validation::default(),
                    )
                    .map_err(|e| {
                        tracing::warn!("JWT validation failed: {:?}", e); // Use tracing for logging
                        AppError::JwtError(e)
                    })?;
                    Some(token_data.claims)
                }
                None => None, // No Authorization header, proceed if path allows
            };

            if let Some(claims) = req_claims {
                req.extensions_mut().insert(claims); // Insert claims into request extensions
                svc.call(req).await
            } else {
                // If there's no claims and the path is not skipped, then it's Unauthorized
                Err(AppError::Unauthorized.into())
            }
        })
    }
}

// --- AdminRoleCheckMiddleware ---

pub struct AdminRoleCheck;

impl<S, B> Transform<S, ServiceRequest> for AdminRoleCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminRoleCheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminRoleCheckMiddleware { service: Rc::new(service) })
    }
}

pub struct AdminRoleCheckMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminRoleCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Using the `forward_ready!` macro
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let claims = req.extensions().get::<Claims>().cloned();

            match claims {
                Some(user_claims) => {
                    if user_claims.role == "admin" {
                        svc.call(req).await // User is admin, proceed
                    } else {
                        // Not an admin
                        Err(AppError::Forbidden.into())
                    }
                }
                None => {
                    // Claims not found, likely because JwtAuth middleware wasn't applied or failed
                    // This implies a configuration error or unauthenticated access to an admin route
                    Err(AppError::Unauthorized.into())
                }
            }
        })
    }
}