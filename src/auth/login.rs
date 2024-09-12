use uuid::Uuid;

use crate::utils::secret::Secret;
use super::pwd::Pwd;

#[derive(Debug, PartialEq, Eq)]
pub struct Login {
    login_id: Uuid,
    username: Secret<String>,
    password: Pwd,
}
