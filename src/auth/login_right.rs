use crate::utils::id::Id;

use super::content_right::ContentRight;

#[derive(Debug, PartialEq, Eq)]
pub struct LoginRight {
    login_id: Id,
    right: ContentRight,
}
