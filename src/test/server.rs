use actix_web::dev::ServerHandle;

use super::{client::TestHttpClient, ports::UsingPort};

pub struct TestServer {
    port: UsingPort,
    http_handle: ServerHandle,
}

impl TestServer {
    pub fn new(port: UsingPort, http_handle: ServerHandle) -> Self {
        Self { port, http_handle }
    }

    pub fn client(&self) -> TestHttpClient {
        TestHttpClient::new(*self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        tokio::spawn(self.http_handle.stop(false));
    }
}
