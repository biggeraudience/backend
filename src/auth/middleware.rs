use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    Error, HttpMessage, web
};
use futures_util::future::{Ready, ok, LocalBoxFuture};
use std::{rc::Rc, task::{Context, Poll}};
use crate::auth::models::Claims;
use crate::error::AppError;
use jsonwebtoken::{decode, DecodingKey, Validation};

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

    forward_ready!(svc);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let next = self.svc.clone();
        let path = req.path().to_owned();

        // public endpoints
        if path.starts_with("/auth/") 
         || path.starts_with("/inquiries") 
         || path.starts_with("/vehicles") {
            return Box::pin(async move { next.call(req).await });
        }

        Box::pin(async move {
            let header = req.headers().get("Authorization").and_then(|h| h.to_str().ok());
            if let Some(hdr) = header {
                if let Some(token) = hdr.strip_prefix("Bearer ") {
                    let secret = req.app_data::<web::Data<String>>().unwrap();
                    let data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &Validation::default()
                    ).map_err(AppError::JwtError)?;
                    req.extensions_mut().insert(data.claims);
                    return next.call(req).await;
                }
            }
            Err(AppError::Unauthorized.into())
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

    forward_ready!(svc);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let next = self.svc.clone();
        Box::pin(async move {
            let is_admin = req
                .extensions()
                .get::<Claims>()
                .map(|c| c.role == "admin")
                .unwrap_or(false);

            if is_admin {
                next.call(req).await
            } else {
                Err(AppError::Forbidden.into())
            }
        })
    }
}
