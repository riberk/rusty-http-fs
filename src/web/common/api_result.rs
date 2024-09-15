use super::api_error::ApiError;

pub type ApiResult<T> = std::result::Result<actix_web::web::Json<T>, ApiError>;
