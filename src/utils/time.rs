use chrono::{DateTime, Utc};

pub trait Time {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Default, Debug)]
pub struct TimeNow {}

impl Time for TimeNow {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}