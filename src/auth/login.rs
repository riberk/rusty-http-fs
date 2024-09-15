use super::pwd::Pwd;
use crate::utils::{id::Id, secret::Secret};

#[derive(Debug, PartialEq, Eq)]
pub struct Login {
    login_id: Id,
    username: Secret<String>,
    password: Pwd,
}
