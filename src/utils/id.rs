use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, derive_more::Deref, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Id(Uuid);

pub trait IdGenerator {
    fn next(&self) -> Id;
}

pub struct DefaultIdGenerator;

impl IdGenerator for DefaultIdGenerator {
    fn next(&self) -> Id {
        Id(Uuid::now_v7())
    }
}
