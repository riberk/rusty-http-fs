use chrono::{DateTime, Utc};

use crate::{utils::id::Id, web::common::serde_chrono::ApiDateTimeSeconds};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: Id,
    pub exp: ApiDateTimeSeconds,
    pub iat: ApiDateTimeSeconds,
    pub jti: Id,
    pub sid: Id,
}

impl RefreshTokenClaims {
    pub fn sub(&self) -> Id {
        self.sub
    }

    pub fn exp(&self) -> DateTime<Utc> {
        *self.exp
    }

    pub fn iat(&self) -> DateTime<Utc> {
        *self.iat
    }

    pub fn jti(&self) -> Id {
        self.jti
    }

    pub fn sid(&self) -> Id {
        self.sid
    }
}
