use std::{fmt::Display, ops::Deref};

use chrono::{serde::ts_milliseconds, DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiDateTime(#[serde(with = "ts_milliseconds")] pub DateTime<Utc>);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiDuration(#[serde(with = "duration_milliseconds")] pub Duration);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiDurationSeconds(#[serde(with = "duration_seconds")] pub Duration);

impl Display for ApiDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for ApiDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for ApiDurationSeconds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<DateTime<Utc>> for ApiDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}

impl From<Duration> for ApiDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl From<Duration> for ApiDurationSeconds {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl Deref for ApiDateTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ApiDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ApiDurationSeconds {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub mod duration_milliseconds {
    use chrono::Duration;
    use core::fmt;
    use serde::{de, ser};

    pub fn serialize<S>(dt: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.num_milliseconds())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(MillisecondsTimestampVisitor)
    }

    pub struct MillisecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for MillisecondsTimestampVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a duration in milliseconds")
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::milliseconds(value))
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value.try_into() {
                Ok(value) => Ok(Duration::milliseconds(value)),
                Err(_) => Err(E::custom(value)),
            }
        }
    }
}

pub mod duration_seconds {
    use chrono::Duration;
    use core::fmt;
    use serde::{de, ser};

    pub fn serialize<S>(dt: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.num_seconds())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(SecondsTimestampVisitor)
    }

    pub struct SecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for SecondsTimestampVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a duration in seconds")
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::seconds(value))
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value.try_into() {
                Ok(value) => Ok(Duration::seconds(value)),
                Err(_) => Err(E::custom(value)),
            }
        }
    }
}