use std::{
    collections::HashSet,
    net::TcpListener,
    ops::Deref,
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub struct Ports {
    used: Arc<Mutex<HashSet<u16>>>,
}

#[derive(Clone)]
pub struct UsingPort {
    value: u16,
    ports: Ports,
}

impl Ports {
    pub fn free(&self, port: u16) {
        let mut used = self.used.lock().unwrap();
        used.remove(&port);
    }

    pub fn acquire(&self) -> UsingPort {
        let mut used = self.used.lock().unwrap();
        let port = loop {
            let port = Self::get_available_port().expect("Unable to find free port");
            if used.insert(port) {
                break port;
            }
        };

        UsingPort {
            value: port,
            ports: self.clone(),
        }
    }

    fn get_available_port() -> Option<u16> {
        match TcpListener::bind(("127.0.0.1", 0)) {
            Ok(b) => Some(b.local_addr().unwrap().port()),
            Err(_) => None,
        }
    }
}

impl Drop for UsingPort {
    fn drop(&mut self) {
        self.ports.free(self.value);
    }
}

impl From<UsingPort> for u16 {
    fn from(value: UsingPort) -> Self {
        value.port()
    }
}

impl Deref for UsingPort {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl UsingPort {
    pub fn port(&self) -> u16 {
        self.value
    }
}
