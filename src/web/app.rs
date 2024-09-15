use actix_service::ServiceFactory;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    web::{self, Data},
    App,
};

use crate::web::common::api_error::ApiError;

use super::{app_data::AppData, trace_id::TraceIdMiddlewareFactory};

pub fn create_app<D: AppData + 'static>(
    app_data: Data<D>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let json_cfg = web::JsonConfig::default().error_handler(|err, req| {
        tracing::info!("json error: {}, request '{:?}'", err, req);
        ApiError::bad_reques()
            .message(err.to_string())
            .build()
            .into()
    });
    App::new()
        .wrap(TraceIdMiddlewareFactory::new((*app_data).clone()))
        .app_data(app_data)
        .app_data(json_cfg)
        .configure(super::routes::configure::<D>)
}
