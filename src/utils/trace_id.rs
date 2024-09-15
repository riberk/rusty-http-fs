use std::{convert::Infallible, str::FromStr};

use actix_http::header::{HeaderValue, TryIntoHeaderValue};
use uuid::Uuid;

use super::id_generator::{DefaultIdGenerator, IdGenerator};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraceId(Uuid);

impl TraceId {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_simple())
    }
}

impl IdGenerator<TraceId> for DefaultIdGenerator {
    fn next_id(&self) -> TraceId {
        TraceId(Uuid::now_v7())
    }
}

impl TryIntoHeaderValue for TraceId {
    type Error = Infallible;

    #[inline]
    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        let id = self.0.as_simple();
        let mut corr_buffer: [u8; uuid::fmt::Simple::LENGTH] = [0; uuid::fmt::Simple::LENGTH];
        id.encode_lower(&mut corr_buffer);
        Ok(HeaderValue::from_bytes(&corr_buffer).unwrap())
    }
}

impl FromStr for TraceId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Uuid::from_str(s) {
            Ok(uuid) => Ok(TraceId(uuid)),
            Err(_) => Err(()),
        }
    }
}
