use chrono::{DateTime, Utc};

use crate::{utils::id::Id, web::common::serde_chrono::ApiDateTimeSeconds};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Id,
    pub exp: ApiDateTimeSeconds,
    pub iat: ApiDateTimeSeconds,
}

impl AccessTokenClaims {
    pub fn sub(&self) -> Id {
        self.sub
    }

    pub fn exp(&self) -> DateTime<Utc> {
        *self.exp
    }

    pub fn iat(&self) -> DateTime<Utc> {
        *self.iat
    }
}
