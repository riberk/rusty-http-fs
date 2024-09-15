use super::pwd_alg::PwdAlg;
use crate::utils::secret::Secret;

#[derive(Debug, PartialEq, Eq)]
pub struct Pwd {
    alg: PwdAlg,
    hash: PwdHash,
}

#[derive(Debug, PartialEq, Eq, derive_more::Deref)]
pub struct PwdHash(Secret<Vec<u8>>);
