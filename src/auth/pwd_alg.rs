#[derive(Debug, PartialEq, Eq)]
pub enum PwdAlg {
    Argon2id(Argon2Params),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Argon2Params {
    m: usize,
    t: usize,
    p: usize,
}
