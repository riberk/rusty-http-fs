use crate::utils::secret::Secret;
use super::pwd_alg::PwdAlg;

#[derive(Debug, PartialEq, Eq)]
pub struct Pwd {
    alg: PwdAlg,
    hash: PwdHash,
}

#[derive(Debug, PartialEq, Eq, derive_more::Deref)]
pub struct PwdHash(Secret<Vec<u8>>);