use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    Error, HttpMessage, web
};
use futures_util::future::{Ready, ok, LocalBoxFuture};
use std::rc::Rc; // Removed unused Context, Poll
use crate::auth::models::Claims;
use crate::error::AppError;
use jsonwebtoken::{decode, DecodingKey, Validation};
use actix_web::http::header::AUTHORIZATION;

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error    = Error;
    type InitError= ();
    type Transform= JwtAuthMiddleware<S>;
    type Future   = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, svc: S) -> Self::Future {
        ok(JwtAuthMiddleware { svc: Rc::new(svc) })
    }
}

pub struct JwtAuthMiddleware<S> {
    svc: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error    = Error;
    type Future   = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(svc); // Macro to implement poll_ready

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let next = self.svc.clone();
        let path = req.path().to_owned();
        let method = req.method().as_str();

        // Define explicit public endpoints that bypass JWT authentication.
        // Adjust these paths if your routes are nested under a common prefix like `/api`.
        let is_public_route = (path.starts_with("/auth/")) || // All auth routes (register, login)
                              (path.starts_with("/vehicles") && method == "GET") || // Public GET for vehicles (e.g., listings)
                              (path.starts_with("/auctions") && method == "GET" && !path.ends_with("/bids")) || // Public GET for auctions (excluding specific bid endpoints)
                              (path.starts_with("/inquiries") && method == "POST"); // Public POST for inquiry submission

        if is_public_route {
            // These routes are genuinely public and do not require authentication
            return Box::pin(async move { next.call(req).await });
        }

        // For all other routes, require JWT authentication
        Box::pin(async move {
            let header = req.headers().get(AUTHORIZATION).and_then(|h| h.to_str().ok());

            if let Some(hdr) = header {
                if let Some(token) = hdr.strip_prefix("Bearer ") {
                    // Access the jwt_secret from app_data
                    let secret = req.app_data::<web::Data<String>>()
                        .ok_or_else(|| Error::from(AppError::InternalServerError("JWT secret not configured in application data.".to_string())))?
                        .as_ref();

                    let data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &Validation::default()
                    ).map_err(|e| Error::from(AppError::JwtError(e)))?; // Convert jsonwebtoken error to AppError

                    req.extensions_mut().insert(data.claims); // Insert claims into request extensions
                    return next.call(req).await;
                }
            }

            // If no valid token or not a Bearer token
            Err(Error::from(AppError::Unauthorized))
        })
    }
}

pub struct AdminRoleCheck;

impl<S, B> Transform<S, ServiceRequest> for AdminRoleCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error    = Error;
    type InitError= ();
    type Transform= AdminRoleCheckMiddleware<S>;
    type Future   = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, svc: S) -> Self::Future {
        ok(AdminRoleCheckMiddleware { svc: Rc::new(svc) })
    }
}

pub struct AdminRoleCheckMiddleware<S> {
    svc: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminRoleCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error    = Error;
    type Future   = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(svc); // Macro to implement poll_ready

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let next = self.svc.clone();
        Box::pin(async move {
            let is_admin = req
                .extensions()
                .get::<Claims>()
                .map(|c| c.role == "admin") // Assuming Claims has a 'role' field
                .unwrap_or(false); // If no claims or no role, default to not admin

            if is_admin {
                next.call(req).await
            } else {
                Err(Error::from(AppError::Forbidden)) // Convert AppError to actix_web::Error
            }
        })
    }
}