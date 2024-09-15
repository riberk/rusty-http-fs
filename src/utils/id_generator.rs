pub trait IdGenerator<T> {
    fn next_id(&self) -> T;
}

pub struct DefaultIdGenerator;
