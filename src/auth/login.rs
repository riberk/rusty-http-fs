use uuid::Uuid;

use super::pwd::Pwd;
use crate::utils::secret::Secret;

#[derive(Debug, PartialEq, Eq)]
pub struct Login {
    login_id: Uuid,
    username: Secret<String>,
    password: Pwd,
}
