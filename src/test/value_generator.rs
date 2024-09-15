use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::utils::{id::Id, id_generator::IdGenerator, trace_id::TraceId};

use super::test_time::DEFAULT_DATE;

pub trait TestValueGenerator {
    fn next_value(&self) -> TestValue;
}

#[derive(Clone, Default)]
pub struct ValueGenerator {
    current: Arc<std::sync::Mutex<u64>>,
}

#[derive(Debug, Clone, Copy, derive_more::Deref)]
pub struct TestValue(u64);

impl TestValueGenerator for ValueGenerator {
    fn next_value(&self) -> TestValue {
        let mut guard = self.current.lock().unwrap();
        *guard += 1;
        TestValue(*guard)
    }
}

pub trait TestValueOptionExtension<T> {
    fn unwrap_or_test_value<G: TestValueGenerator>(self, generator: &mut G) -> T;
}

impl<T> TestValueOptionExtension<T> for Option<T>
where
    T: From<TestValue>,
{
    fn unwrap_or_test_value<G: TestValueGenerator>(self, generator: &mut G) -> T {
        self.unwrap_or_else(|| generator.next_value().into())
    }
}

macro_rules! impl_num {
    ($type:ty) => {
        impl From<TestValue> for $type {
            fn from(value: TestValue) -> Self {
                value.0 as $type
            }
        }
    };
}

impl_num!(i8);
impl_num!(u8);
impl_num!(i16);
impl_num!(u16);
impl_num!(i32);
impl_num!(u32);
impl_num!(i64);
impl_num!(u64);
impl_num!(i128);
impl_num!(u128);
impl_num!(f32);
impl_num!(f64);

impl From<TestValue> for DateTime<Utc> {
    fn from(value: TestValue) -> Self {
        *DEFAULT_DATE + chrono::Duration::seconds(value.0 as i64)
    }
}

impl From<TestValue> for Uuid {
    fn from(value: TestValue) -> Self {
        Uuid::from_u128(*value as u128)
    }
}

impl From<TestValue> for Id {
    fn from(value: TestValue) -> Self {
        Self::from_uuid(value.into())
    }
}

impl From<TestValue> for TraceId {
    fn from(value: TestValue) -> Self {
        Self::from_uuid(value.into())
    }
}

impl<T: TestValueGenerator> IdGenerator<Id> for T {
    fn next_id(&self) -> Id {
        self.next_value().into()
    }
}

impl<T: TestValueGenerator> IdGenerator<TraceId> for T {
    fn next_id(&self) -> TraceId {
        self.next_value().into()
    }
}
