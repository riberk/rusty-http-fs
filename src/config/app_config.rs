use config::Environment;
use serde::{Deserialize, Serialize};

use crate::utils::secret::Secret;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    secrets: SecretsConfig,
}

impl AppConfig {
    pub fn secrets(&self) -> &SecretsConfig {
        &self.secrets
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecretsConfig {
    tokens: TokenSecretsConfig,
}

impl SecretsConfig {
    pub fn tokens(&self) -> &TokenSecretsConfig {
        &self.tokens
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenSecretsConfig {
    access_secret: Secret<String>,
    refresh_secret: Secret<String>,
}

impl TokenSecretsConfig {
    pub fn access_secret(&self) -> &str {
        &self.access_secret
    }

    pub fn refresh_secret(&self) -> &str {
        &self.refresh_secret
    }
}

pub static ENVIRONMENT_PREFIX: &str = "RHFS";
pub static ENVIRONMENT_SEPARATOR: &str = "__";

pub fn env_source() -> Environment {
    Environment::with_prefix(ENVIRONMENT_PREFIX)
        .try_parsing(true)
        .separator(ENVIRONMENT_SEPARATOR)
}
