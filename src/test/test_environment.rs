use std::sync::Arc;

use config::Config;

use crate::config::app_config::{env_source, AppConfig};

#[derive(Clone)]
pub struct TestEnvironment {
    config: Arc<AppConfig>,
    number: usize,
}

impl TestEnvironment {
    pub async fn make(number: usize) -> Self {
        let builder = Config::builder()
            .add_source(config::File::with_name("config/test").required(true))
            .add_source(env_source());
        let config = builder.build().unwrap();
        let config = config.try_deserialize::<AppConfig>().unwrap();
        TestEnvironment {
            number,
            config: Arc::new(config),
        }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn config(&self) -> &Arc<AppConfig> {
        &self.config
    }
}
