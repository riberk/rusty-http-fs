use std::{
    future::{ready, Ready},
    sync::Arc,
};

use actix_service::{forward_ready, Service, ServiceFactory, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    web::{Data, ServiceConfig},
    App,
};
use serde::de::DeserializeOwned;
use tracing::{
    instrument::{WithDispatch, WithSubscriber},
    subscriber::with_default,
    Subscriber,
};

use crate::{
    test::{get_free_port, ports::UsingPort},
    web::{
        app::{self},
        app_data::DefaultAppData,
        common::api_error::ApiError,
    },
};

use super::{
    client::TestHttpResponse,
    server::TestServer,
    test_context::{TestContext, TestContextAppFactory},
};

pub trait RunServer {
    #[allow(async_fn_in_trait)]
    async fn run_server(&self) -> TestServer {
        self.run_server_with(|_| {}).await
    }

    #[allow(async_fn_in_trait)]
    async fn run_server_with(
        &self,
        configure: impl Fn(&mut ServiceConfig) + Send + 'static + Clone,
    ) -> TestServer;
}

impl RunServer for TestContext {
    async fn run_server_with(
        &self,
        configure: impl Fn(&mut ServiceConfig) + Send + 'static + Clone,
    ) -> TestServer {
        let port: UsingPort = get_free_port();
        let factory = self.factory();
        // let configure = configure;
        let server = actix_web::HttpServer::new(move || {
            factory.clone().make_app().configure(|cfg| configure(cfg))
        })
        .workers(4)
        .bind(("127.0.0.1", *port))
        .unwrap()
        .run();

        let http_handle = server.handle();
        tokio::task::spawn(server.with_subscriber(self.logs().make_subscriber()));
        tracing::info!(port = *port, "Server running");
        TestServer::new(port, http_handle)
    }
}

struct SetSubscriberMidlewareFactory<L>(Arc<L>);

impl<S, L> Transform<S, ServiceRequest> for SetSubscriberMidlewareFactory<L>
where
    S: Service<ServiceRequest>,
    S::Future: 'static,
    L: Subscriber + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = SetSubscriberMidleware<S, L>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SetSubscriberMidleware {
            service,
            subscriber: self.0.clone(),
        }))
    }
}

pub struct SetSubscriberMidleware<S, L> {
    service: S,
    subscriber: Arc<L>,
}

impl<S, L> Service<ServiceRequest> for SetSubscriberMidleware<S, L>
where
    S: Service<ServiceRequest>,
    S::Future: 'static,
    L: Subscriber + Send + Sync + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = WithDispatch<S::Future>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        with_default(self.subscriber.clone(), || self.service.call(req))
            .with_subscriber(self.subscriber.clone())
    }
}

pub trait AppFactory {
    fn make_app(
        self,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<impl actix_http::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    >;
}

impl AppFactory for TestContextAppFactory {
    fn make_app(
        self,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<impl actix_http::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        let app = app::create_app(Data::new(DefaultAppData::new(
            self.time.clone(),
            self.value_generator.clone(),
            self.value_generator.clone(),
        )));
        let subscriber = self.logs.make_subscriber();
        app.wrap(SetSubscriberMidlewareFactory(subscriber.into()))
    }
}

pub trait TestResponse {
    fn api_result<T: DeserializeOwned>(&self) -> Result<T, ApiError>;
    fn unwrap<T: DeserializeOwned>(&self) -> T;
    fn unwrap_err(&self) -> ApiError;
    fn unwrap_custom_err<E: DeserializeOwned>(&self) -> E;
}

impl TestResponse for TestHttpResponse {
    /// Returns
    ///
    /// Ok, if 2xx
    ///
    /// Err, if another
    fn api_result<T: DeserializeOwned>(&self) -> Result<T, ApiError> {
        self.result()
    }

    fn unwrap<T: DeserializeOwned>(&self) -> T {
        self.api_result().unwrap()
    }

    fn unwrap_err(&self) -> ApiError {
        self.unwrap_custom_err()
    }

    fn unwrap_custom_err<E: DeserializeOwned>(&self) -> E {
        self.result::<(), E>().unwrap_err()
    }
}
