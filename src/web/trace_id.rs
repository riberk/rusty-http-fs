use std::{
    future::{ready, Ready},
    sync::Arc,
};

use actix_http::{header::TryIntoHeaderValue, HttpMessage};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use tracing::Instrument;

use crate::utils::id_generator::IdGenerator;

use super::app_data::AppData;

#[derive(Debug, Default)]
pub struct TraceIdMiddlewareFactory<D>(Arc<D>);

impl<D> TraceIdMiddlewareFactory<D> {
    pub fn new(data: Arc<D>) -> Self {
        Self(data)
    }
}

impl<S, B, D: AppData> Transform<S, ServiceRequest> for TraceIdMiddlewareFactory<D>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TraceIdMiddleware<S, D>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TraceIdMiddleware {
            service,
            app_data: self.0.clone(),
        }))
    }
}

pub struct TraceIdMiddleware<S, D> {
    service: S,
    app_data: Arc<D>,
}

#[allow(clippy::declare_interior_mutable_const)]
pub const HEADER_NAME: actix_http::header::HeaderName =
    actix_http::header::HeaderName::from_static("x-traceid");

impl<S, B, D: AppData> Service<ServiceRequest> for TraceIdMiddleware<S, D>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let trace_id = self.app_data.trace_id().next_id();
        req.extensions_mut().insert(trace_id);
        let span = tracing::info_span!("req", trace_id = %trace_id);
        let path = req.path().to_owned();
        let method = req.method().as_ref().to_owned();

        let started = std::time::Instant::now();
        {
            let _guard = span.enter();
            tracing::info!(path = path, method = method, "request starting...");
        }
        let fut = self.service.call(req);
        Box::pin(
            async move {
                let mut res = fut.await?;
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                res.headers_mut()
                    .insert(HEADER_NAME, trace_id.try_into_value().unwrap());
                let status = res.status().as_u16();
                tracing::info!(
                    path = path,
                    method = method,
                    status = status,
                    elapsed_ms = elapsed_ms,
                    "request finished on {} ms",
                    elapsed_ms
                );
                Ok(res)
            }
            .instrument(span),
        )
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::{
        test::*,
        utc,
        utils::trace_id::TraceId,
        web::{common::api_result::ApiResult, trace_id},
    };
    use actix_web::web;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    #[test]
    fn trace_id() {
        test(|ctx| async move {
            ctx.logs().write_always();
            const MESSAGE: &str = "feaugfhsreughvserbvushrfkjhrdsg";

            async fn route(trace_id: web::ReqData<TraceId>) -> ApiResult<TraceId> {
                tracing::info!("{}", MESSAGE);
                Ok(web::Json(trace_id.into_inner()))
            }

            let server = ctx
                .run_server_with(|cfg| {
                    cfg.route("/test/trace_id", web::get().to(route));
                })
                .await;

            let now = utc!(2023, 12, 1, 2, 3, 4, 125);
            ctx.time().set(now);

            // act
            ctx.logs().clear();
            let response = server.client().get("/test/trace_id").send().await;

            // assert
            let response_trace_id = response.unwrap::<TraceId>();
            let header_trace_id: TraceId = response
                .headers
                .get(trace_id::HEADER_NAME)
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .unwrap();

            let log_trace_id = ctx
                .logs()
                .get(|e| e.message() == MESSAGE)
                .must_have_span_field_value::<TraceId>("req1", "trace_id");

            assert_eq!(response_trace_id, header_trace_id);
            assert_eq!(response_trace_id, log_trace_id);
        });
    }
}
