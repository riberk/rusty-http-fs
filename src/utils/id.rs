use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::id_generator::{DefaultIdGenerator, IdGenerator};

#[derive(
    Debug, Clone, Copy, derive_more::Deref, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct Id(Uuid);

impl Id {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }
}

impl IdGenerator<Id> for DefaultIdGenerator {
    fn next_id(&self) -> Id {
        Id(Uuid::now_v7())
    }
}
