use std::{
    future::{ready, Ready},
    str::from_utf8,
    sync::Arc,
};

use actix_service::{forward_ready, Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    HttpMessage,
};
use jsonwebtoken::TokenData;
use tracing::{instrument::Instrumented, Instrument};

use crate::auth::{
    principal::Principal,
    tokens::{access_token_claims::AccessTokenClaims, encoder::JwtTokenDecoder},
};

pub struct JwtAuthenticationMiddlewareFactory(Arc<JwtTokenDecoder<AccessTokenClaims>>);

impl JwtAuthenticationMiddlewareFactory {
    pub fn new<T: Into<Arc<JwtTokenDecoder<AccessTokenClaims>>>>(decoder: T) -> Self {
        Self(decoder.into())
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthenticationMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = JwtAuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthenticationMiddleware {
            service,
            decoder: self.0.clone(),
        }))
    }
}

pub struct JwtAuthenticationMiddleware<S> {
    service: S,
    decoder: Arc<JwtTokenDecoder<AccessTokenClaims>>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = S::Response;
    type Error = actix_web::Error;
    type Future = Instrumented<S::Future>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let span = match parse_token(&self.decoder, &req) {
            Some(token) => {
                tracing::info!("User '{}' has been authenticated", token.claims.sub());
                let principal = Principal::new(token.claims.sub());
                let id = principal.id();
                req.extensions_mut().insert(principal);
                tracing::info_span!("principal", id = %id)
            }
            None => {
                tracing::info!("User hasn't been authenticated");
                tracing::info_span!("principal")
            }
        };
        span.in_scope(|| self.service.call(req)).instrument(span)
    }
}

fn parse_token(
    decoder: &JwtTokenDecoder<AccessTokenClaims>,
    req: &ServiceRequest,
) -> Option<TokenData<AccessTokenClaims>> {
    const AUTH_HEADER_PREFIX: &str = "Bearer ";

    req.headers()
        .get(actix_http::header::AUTHORIZATION)
        .map(|h| h.as_bytes())
        .map(from_utf8)
        .and_then(Result::ok)
        .and_then(|t| t.strip_prefix(AUTH_HEADER_PREFIX))
        .and_then(|t| decoder.decode(t).ok())
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::{test::*, utc, utils::id::Id, web::common::api_result::ApiResult};
    use actix_web::web::{self, Json, ServiceConfig};
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    use test_subscriber::{LogField, SpanData};

    const LOG_MESSAGE: &str = "fsejfosdenrgviunsdouvnslrzvnsdkjnbvisfdnbds";
    async fn route(principal: Option<web::ReqData<Principal>>) -> ApiResult<Option<Principal>> {
        tracing::info!("{}", LOG_MESSAGE);
        Ok(Json(principal.map(|v| v.into_inner())))
    }

    fn configure(cfg: &mut ServiceConfig) {
        cfg.route("/test", web::get().to(route));
    }

    #[test]
    fn principal_is_set_with_good_jwt() {
        test(|ctx| async move {
            // arrange
            let server = ctx.run_server_with(configure).await;
            ctx.time().set(utc!(2000));
            let principal_id = Id::from_u128(0x999_999);
            let token = ctx
                .access_token_encoder()
                .encode(&AccessTokenClaims {
                    sub: principal_id,
                    exp: utc!(2100).into(),
                    iat: utc!(1900).into(),
                })
                .unwrap();

            // act
            let response = server
                .client()
                .get("/test")
                .access_token(&token)
                .send()
                .await;

            // assert
            let principal = response.unwrap::<Option<Principal>>();
            let principal = principal.ok_or("Principal is None").unwrap();
            let expected = Principal::new(principal_id);
            assert_eq!(principal, expected);

            let header_trace_id = response.trace_id();
            let log_entry = ctx.logs().get(|e| e.message() == LOG_MESSAGE);
            let spans = log_entry.spans();
            let expected_spans = [
                SpanData::new(
                    "principal",
                    &[LogField::new("id", principal_id.to_string())],
                ),
                SpanData::new(
                    "req",
                    &[LogField::new("trace_id", header_trace_id.to_string())],
                ),
            ];
            assert_eq!(spans, expected_spans);
        });
    }

    #[test]
    fn principal_is_not_set_if_jwt_has_expired() {
        test(|ctx| async move {
            // arrange
            let server = ctx.run_server_with(configure).await;

            ctx.time().set(utc!(2000));
            let principal_id = Id::from_u128(1);
            let token = ctx
                .access_token_encoder()
                .encode(&AccessTokenClaims {
                    sub: principal_id,
                    exp: utc!(1999, 12, 31, 23, 59, 59).into(),
                    iat: utc!(1900).into(),
                })
                .unwrap();

            // act
            let principal = server
                .client()
                .get("/test")
                .access_token(&token)
                .send()
                .await
                .unwrap::<Option<Principal>>();

            // assert
            assert!(
                principal.is_none(),
                "Principal is set, but expected none: {:?}",
                principal
            );
        });
    }

    #[test]
    fn principal_is_not_set_if_jwt_has_invalid_signature() {
        test(|ctx| async move {
            // arrange
            let server = ctx.run_server_with(configure).await;

            ctx.time().set(utc!(2000));
            let principal_id = Id::from_u128(1);
            let token = ctx
                .refresh_token_encoder()
                .encode(&AccessTokenClaims {
                    sub: principal_id,
                    exp: utc!(2999).into(),
                    iat: utc!(1900).into(),
                })
                .unwrap();

            // act
            let principal = server
                .client()
                .get("/test")
                .access_token(&token)
                .send()
                .await
                .unwrap::<Option<Principal>>();

            // assert
            assert!(
                principal.is_none(),
                "Principal is set, but expected none: {:?}",
                principal
            );
        });
    }
}
