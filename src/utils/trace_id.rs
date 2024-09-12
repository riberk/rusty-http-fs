#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TraceId(pub uuid::Uuid);

impl TraceId {
    pub fn generate() -> TraceId {
        Self(uuid::Uuid::now_v7())
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_simple())
    }
}