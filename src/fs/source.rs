use std::path::PathBuf;

use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub struct Source {
    id: Uuid,
    path: PathBuf,
}


