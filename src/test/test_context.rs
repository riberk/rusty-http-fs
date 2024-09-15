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

    pub fn factory(&self) -> TestContextAppFactory {
        TestContextAppFactory {
            time: self.time.clone(),
            value_generator: self.value_generator.clone(),
            logs: self.logs().clone(),
        }
    }

    pub fn logs(&self) -> &LogCollector {
        &self.logs
    }

    pub fn generator(&mut self) -> &mut ValueGenerator {
        &mut self.value_generator
    }

    pub fn time(&self) -> &TestTime {
        &self.time
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

#[derive(Clone)]
pub struct TestContextAppFactory {
    pub time: TestTime,
    pub value_generator: ValueGenerator,
    pub logs: LogCollector,
}
