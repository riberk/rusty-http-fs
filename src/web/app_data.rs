use crate::utils::{
    id::Id,
    id_generator::{self, IdGenerator},
    time::{self, Time},
    trace_id::TraceId,
};

pub trait AppData {
    type Time: Time;
    type TraceIdGenerator: IdGenerator<TraceId>;
    type IdGenerator: IdGenerator<Id>;

    fn time(&self) -> &Self::Time;
    fn trace_id(&self) -> &Self::TraceIdGenerator;
    fn id(&self) -> &Self::IdGenerator;
}

pub struct DefaultAppData<Time, TraceIdGenerator, IdGenerator> {
    time: Time,
    trace_id: TraceIdGenerator,
    id: IdGenerator,
}

impl<Time, TraceIdGenerator, IdGenerator> DefaultAppData<Time, TraceIdGenerator, IdGenerator> {
    pub fn new(time: Time, trace_id: TraceIdGenerator, id: IdGenerator) -> Self {
        Self { time, trace_id, id }
    }
}

impl<
        Time: time::Time,
        TraceIdGenerator: id_generator::IdGenerator<TraceId>,
        IdGenerator: id_generator::IdGenerator<Id>,
    > AppData for DefaultAppData<Time, TraceIdGenerator, IdGenerator>
{
    type Time = Time;
    type TraceIdGenerator = TraceIdGenerator;
    type IdGenerator = IdGenerator;

    fn time(&self) -> &Self::Time {
        &self.time
    }

    fn trace_id(&self) -> &Self::TraceIdGenerator {
        &self.trace_id
    }

    fn id(&self) -> &Self::IdGenerator {
        &self.id
    }
}
