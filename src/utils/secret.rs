use std::fmt::Debug;

#[derive(PartialEq, Eq, derive_more::Deref, serde::Serialize, serde::Deserialize)]
pub struct Secret<T>(T);

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Secret").finish()
    }
}
