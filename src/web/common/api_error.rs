use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    Conflict,
    TooManyRequests,
    UnexpectedError,
}

impl ErrorCode {
    pub fn http_status(&self) -> StatusCode {
        match self {
            ErrorCode::BadRequest => StatusCode::BAD_REQUEST,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::PaymentRequired => StatusCode::PAYMENT_REQUIRED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ErrorCode,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

pub struct ApiErrorBuilder {
    code: ErrorCode,
    message: Option<String>,
    details: Option<String>,
}

impl ApiErrorBuilder {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            message: None,
            details: None,
        }
    }

    pub fn message(mut self, v: String) -> Self {
        self.message = Some(v);
        self
    }

    pub fn details(mut self, v: String) -> Self {
        self.details = Some(v);
        self
    }

    pub fn build(self) -> ApiError {
        ApiError {
            code: self.code,
            message: self.message,
            details: self.details,
        }
    }
}

impl From<ApiErrorBuilder> for ApiError {
    fn from(value: ApiErrorBuilder) -> Self {
        value.build()
    }
}

impl ApiError {
    pub fn builder(code: ErrorCode) -> ApiErrorBuilder {
        ApiErrorBuilder::new(code)
    }

    pub fn bad_reques() -> ApiErrorBuilder {
        Self::builder(ErrorCode::BadRequest)
    }

    pub fn unauthorized() -> ApiErrorBuilder {
        Self::builder(ErrorCode::Unauthorized)
    }

    pub fn payment_required() -> ApiErrorBuilder {
        Self::builder(ErrorCode::PaymentRequired)
    }

    pub fn forbidden() -> ApiErrorBuilder {
        Self::builder(ErrorCode::Forbidden)
    }

    pub fn not_found() -> ApiErrorBuilder {
        Self::builder(ErrorCode::NotFound)
    }

    pub fn conflict() -> ApiErrorBuilder {
        Self::builder(ErrorCode::Conflict)
    }

    pub fn too_many_requests() -> ApiErrorBuilder {
        Self::builder(ErrorCode::TooManyRequests)
    }

    pub fn unexpected() -> ApiErrorBuilder {
        Self::builder(ErrorCode::UnexpectedError)
    }

    fn make_details(details: Option<String>) -> Option<String> {
        details.and_then(|s| if cfg!(test) { Some(s) } else { None })
    }

    fn to_details<T: Debug>(error: T) -> Option<String> {
        if cfg!(test) {
            Some(format!("{:?}", error))
        } else {
            None
        }
    }

    fn from_details<T: Debug>(error: T) -> Self {
        ApiError {
            code: ErrorCode::UnexpectedError,
            message: None,
            details: Self::to_details(error),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message.as_deref();
        match message {
            Some(v) => write!(f, "{}: '{}'", self.code, v),
            None => write!(f, "{}", self.code),
        }
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.code.http_status()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());
        let mime = mime::APPLICATION_JSON.try_into_value().unwrap();
        res.headers_mut()
            .insert(actix_web::http::header::CONTENT_TYPE, mime);
        let body = match serde_json::to_vec(self) {
            Ok(body) => body,
            Err(e) => {
                tracing::error!("error serialization error: {}", e);
                b"{\"code\":\"unexpected_error\",\"message\":\"Serialization error\"}".to_vec()
            }
        };
        res.set_body(actix_web::body::BoxBody::new(body))
    }
}
