use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::id_generator::{DefaultIdGenerator, IdGenerator};

#[derive(
    Debug, Clone, Copy, derive_more::Deref, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct Id(Uuid);

impl Id {
    #[inline]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }

    #[inline]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }

    #[inline]
    pub const fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_simple())
    }
}

impl FromStr for Id {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Uuid::from_str(s) {
            Ok(uuid) => Ok(Self(uuid)),
            Err(_) => Err(()),
        }
    }
}

impl IdGenerator<Id> for DefaultIdGenerator {
    fn next_id(&self) -> Id {
        Id(Uuid::now_v7())
    }
}
