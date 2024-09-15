use actix_service::ServiceFactory;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    web::{self, Data},
    App,
};

use crate::{auth::tokens::encoder::TokensEncDec, web::common::api_error::ApiError};

use super::{
    app_data::AppData, auth::jwt_auth_middleware::JwtAuthenticationMiddlewareFactory,
    trace_id::TraceIdMiddlewareFactory,
};

pub fn create_app<D: AppData + 'static>(
    app_data: Data<D>,
    token_encoders: TokensEncDec,
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
    let access_decoder = Data::new(token_encoders.access.decoder);
    App::new()
        .wrap(JwtAuthenticationMiddlewareFactory::new(
            (*access_decoder).clone(),
        ))
        .wrap(TraceIdMiddlewareFactory::new((*app_data).clone()))
        .app_data(app_data)
        .app_data(json_cfg)
        .app_data(Data::new(token_encoders.access.encoder))
        .app_data(access_decoder)
        .app_data(Data::new(token_encoders.refresh.encoder))
        .app_data(Data::new(token_encoders.refresh.decoder))
        .configure(super::routes::configure::<D>)
}
