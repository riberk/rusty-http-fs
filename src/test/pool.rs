use std::{future::Future, ops::Deref};

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub struct Pool<T> {
    sender: UnboundedSender<PoolValue<T>>,
    state: Mutex<State<T>>,

    /// How many items can be received from pool
    capacity: usize,

    number_start_from: usize,
}

struct State<T> {
    recv: UnboundedReceiver<PoolValue<T>>,

    /// How many items have been received from pool
    count: usize,
}

impl<T> Pool<T> {
    pub fn new(max_capacity: usize, number_start_from: usize) -> Pool<T> {
        let (sender, recv) = tokio::sync::mpsc::unbounded_channel();
        Pool {
            sender,
            capacity: max_capacity,
            number_start_from,
            state: State { recv, count: 0 }.into(),
        }
    }

    pub async fn acquire<F, Fut>(&self, f: F) -> PoolValue<T>
    where
        F: FnOnce(usize) -> Fut,
        Fut: Future<Output = T>,
    {
        let mut state = self.state.lock().await;
        if let Ok(value) = state.recv.try_recv() {
            value
        } else if state.count >= self.capacity {
            state.recv.recv().await.unwrap()
        } else {
            let number = state.count + self.number_start_from;
            state.count += 1;
            let value = f(number).await;
            PoolValue {
                value: Some(value),
                sender: self.sender.clone(),
                number,
            }
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new(16, 0)
    }
}

pub struct PoolValue<T> {
    sender: UnboundedSender<PoolValue<T>>,
    value: Option<T>,
    number: usize,
}

impl<T> Drop for PoolValue<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            let _ = self.sender.send(PoolValue {
                sender: self.sender.clone(),
                number: self.number,
                value: Some(value),
            });
        }
    }
}

impl<T> Deref for PoolValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> PoolValue<T> {
    pub fn number(&self) -> usize {
        self.number
    }
}
