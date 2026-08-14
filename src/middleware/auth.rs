use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::task::{Context, Poll};

use crate::models::{ApiError, Claims};

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareService { service })
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                if h.starts_with("Bearer ") {
                    Some(h[7..].to_string())
                } else {
                    None
                }
            });

        let jwt_secret = req
            .app_data::<web::Data<String>>()
            .map(|s| s.get_ref().clone())
            .unwrap_or_default();

        if let Some(token) = token {
            let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
            let validation = Validation::default();

            match decode::<Claims>(&token, &decoding_key, &validation) {
                Ok(token_data) => {
                    req.extensions_mut().insert(token_data.claims);
                    let fut = self.service.call(req);
                    Box::pin(async move {
                        let res = fut.await?;
                        Ok(res.map_into_left_body())
                    })
                }
                Err(_) => {
                    let res = req.into_response(
                        HttpResponse::Unauthorized()
                            .json(ApiError::new("Invalid or expired token"))
                            .map_into_right_body(),
                    );
                    Box::pin(async { Ok(res) })
                }
            }
        } else {
            let res = req.into_response(
                HttpResponse::Unauthorized()
                    .json(ApiError::new("Authorization token required"))
                    .map_into_right_body(),
            );
            Box::pin(async { Ok(res) })
        }
    }
}
