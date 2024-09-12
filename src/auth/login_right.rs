use uuid::Uuid;

use super::content_right::ContentRight;

#[derive(Debug, PartialEq, Eq)]
pub struct LoginRight {
    login_id: Uuid,
    right: ContentRight,
}
