use crate::auth::tokens::access_token_claims::AccessTokenClaims;
use crate::auth::tokens::encoder::{EncDecPair, JwtTokenEncoder};

use super::test_environment::TestEnvironment;
use super::test_subscriber::LogCollector;
use super::{pool::PoolValue, test_time::TestTime};

use super::value_generator::ValueGenerator;

pub struct TestContext {
    time: TestTime,
    value_generator: ValueGenerator,
    environment: PoolValue<TestEnvironment>,
    logs: LogCollector,
}

impl TestContext {
    pub fn env(&self) -> &TestEnvironment {
        &self.environment
    }

    pub fn access_token_encoder(&self) -> JwtTokenEncoder<AccessTokenClaims> {
        EncDecPair::from_secret(self.env().config().secrets().tokens().access_secret()).encoder
    }

    pub fn refresh_token_encoder(&self) -> JwtTokenEncoder<AccessTokenClaims> {
        EncDecPair::from_secret(self.env().config().secrets().tokens().refresh_secret()).encoder
    }

    pub fn logs(&self) -> &LogCollector {
        &self.logs
    }

    pub fn value_generator(&self) -> &ValueGenerator {
        &self.value_generator
    }

    pub fn time(&self) -> &TestTime {
        &self.time
    }

    pub fn enable_log_output(&self) {
        _ = tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_max_level(tracing::Level::TRACE)
            .try_init();
    }
}

impl PoolValue<TestEnvironment> {
    pub async fn start_test(self, logs: LogCollector) -> TestContext {
        let value_generator: ValueGenerator = Default::default();
        TestContext {
            value_generator: value_generator.clone(),
            time: TestTime::default(),
            environment: self,
            logs,
        }
    }
}
