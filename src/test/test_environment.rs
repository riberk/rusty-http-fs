#[derive(Clone)]
pub struct TestEnvironment {
    number: usize,
}

impl TestEnvironment {
    pub async fn make(number: usize) -> Self {
        TestEnvironment { number }
    }

    pub fn number(&self) -> usize {
        self.number
    }
}
