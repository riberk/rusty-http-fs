use serde::{Deserialize, Serialize};

use crate::utils::id::Id;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Principal {
    id: Id,
}

impl Principal {
    pub fn new(id: Id) -> Self {
        Self { id }
    }

    pub fn id(&self) -> Id {
        self.id
    }
}
