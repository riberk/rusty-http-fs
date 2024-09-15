use std::sync::{Arc, LazyLock, RwLock};

use chrono::{DateTime, Utc};

pub static DEFAULT_DATE: LazyLock<DateTime<Utc>> = LazyLock::new(|| utc!(2000));

#[derive(Clone)]
pub struct TestTime {
    now: Arc<RwLock<DateTime<Utc>>>,
}

impl TestTime {
    pub fn new(date: DateTime<Utc>) -> TestTime {
        Self {
            now: Arc::new(RwLock::new(date)),
        }
    }

    pub fn set(&self, date: DateTime<Utc>) {
        *(self.now.write().unwrap()) = date
    }

    pub fn set_now(&self) {
        self.set(Utc::now());
    }
}

impl Time for TestTime {
    fn now(&self) -> DateTime<Utc> {
        *(self.now.read().unwrap())
    }
}

impl Default for TestTime {
    fn default() -> Self {
        TestTime::new(*DEFAULT_DATE)
    }
}
#[macro_export]
macro_rules! utc {
    () => {{
        use chrono::TimeZone;
        chrono::Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).unwrap()
    }};

    ($year: expr) => {{
        use chrono::TimeZone;
        chrono::Utc.with_ymd_and_hms($year, 1, 1, 0, 0, 0).unwrap()
    }};

    ($year: expr, $month: expr) => {{
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms($year, $month, 1, 0, 0, 0)
            .unwrap()
    }};

    ($year: expr, $month: expr, $day: expr) => {{
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms($year, $month, $day, 0, 0, 0)
            .unwrap()
    }};

    ($year: expr, $month: expr, $day: expr, $hour: expr) => {{
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms($year, $month, $day, $hour, 0, 0)
            .unwrap()
    }};

    ($year: expr, $month: expr, $day: expr, $hour: expr, $minute: expr) => {{
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms($year, $month, $day, $hour, $minute, 0)
            .unwrap()
    }};

    ($year: expr, $month: expr, $day: expr, $hour: expr, $minute: expr, $second: expr) => {{
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms($year, $month, $day, $hour, $minute, $second)
            .unwrap()
    }};

    ($year: expr, $month: expr, $day: expr, $hour: expr, $minute: expr, $second: expr, $ms: expr) => {{
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms($year, $month, $day, $hour, $minute, $second)
            .unwrap()
            .checked_add_signed(chrono::Duration::milliseconds($ms as i64))
            .unwrap()
            .into()
    }};
}
#[allow(unused_imports)]
pub(crate) use utc;

use crate::utils::time::Time;
