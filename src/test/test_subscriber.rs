use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    str::FromStr,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    Layer,
};

#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: Level,
    target: String,
    message: String,
    fields: Vec<LogField>,
    spans: Vec<SpanData>,
}

impl LogEntry {
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn fields(&self) -> &[LogField] {
        &self.fields
    }

    pub fn field<N: AsRef<str>>(&self, name: N) -> Option<&LogField> {
        self.fields.iter().find(|f| f.name() == name.as_ref())
    }

    pub fn span_field<N: AsRef<str>>(&self, span_name: &str, field_name: N) -> Option<&LogField> {
        self.spans
            .iter()
            .find(|s| s.name() == span_name)
            .into_iter()
            .flat_map(|s| s.fields())
            .find(|f| f.name() == field_name.as_ref())
    }

    pub fn must_have_field_value<T: FromStr<Err: Debug>>(&self, name: &str) -> T {
        self.field(name).unwrap().value().parse().unwrap()
    }

    pub fn must_have_span_field_value<T: FromStr<Err: Debug>>(
        &self,
        span_name: &str,
        field_name: &str,
    ) -> T {
        self.span_field(span_name, field_name)
            .unwrap()
            .value()
            .parse()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogField {
    name: String,
    value: String,
}

impl Display for LogField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name(), self.value())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanData {
    name: String,
    fields: Vec<LogField>,
}

impl SpanData {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[LogField] {
        &self.fields
    }
}

impl LogField {
    pub fn new<'a, N: Into<Cow<'a, str>>, V: Into<Cow<'a, str>>>(name: N, value: V) -> Self {
        Self {
            name: name.into().into_owned(),
            value: value.into().into_owned(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

struct FormatFields<'a>(&'a [LogField]);

impl Display for FormatFields<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fields = self.0.iter();
        if let Some(field) = fields.next() {
            write!(f, "{}", field)?;
            for field in fields {
                write!(f, ", {}", field)?;
            }
        }
        Ok(())
    }
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;
        let level = match self.level {
            Level::TRACE => "T".bright_blue(),
            Level::DEBUG => "D".blue(),
            Level::INFO => "I".green(),
            Level::WARN => "W".yellow(),
            Level::ERROR => "E".red(),
        };

        write!(
            f,
            "{} [{}] {} [{}]",
            level.bold().italic(),
            self.target.dimmed(),
            self.message(),
            FormatFields(self.fields())
        )?;

        for (idx, span) in self.spans.iter().enumerate() {
            writeln!(f)?;
            for _ in 0..((idx + 1) * 2) {
                write!(f, " ")?;
            }
            write!(
                f,
                "{}: [{}]",
                span.name.bold().dimmed(),
                FormatFields(&span.fields)
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LogCollector {
    data: Arc<Mutex<Option<LogCollectorData>>>,
}

impl Default for LogCollector {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Some(Default::default()))),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogCollectorData {
    logs: Vec<LogEntry>,
    write_behaviour: WriteBehaviour,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum WriteBehaviour {
    #[default]
    OnError,
    Always,
    Never,
}

impl LogCollectorData {
    pub fn logs(&self) -> impl Iterator<Item = &LogEntry> {
        self.logs.iter()
    }

    pub fn write_behaviour(&self) -> WriteBehaviour {
        self.write_behaviour
    }
}

impl LogCollector {
    fn with_guard<T>(&self, f: impl FnOnce(&mut LogCollectorData) -> T) -> Option<T> {
        let mut guard = self.data.lock().unwrap_or_else(|e| e.into_inner());
        guard.as_mut().map(f)
    }

    pub fn clear(&self) {
        self.with_guard(|d| d.logs.clear());
    }

    pub fn set_write_behaviour(&self, b: WriteBehaviour) {
        self.with_guard(|d| d.write_behaviour = b);
    }

    pub fn write_always(&self) {
        self.set_write_behaviour(WriteBehaviour::Always)
    }

    pub fn write_never(&self) {
        self.set_write_behaviour(WriteBehaviour::Never)
    }

    pub fn write_on_error(&self) {
        self.set_write_behaviour(WriteBehaviour::OnError)
    }

    fn push(&self, entry: LogEntry) {
        self.with_guard(|d| d.logs.push(entry));
    }

    pub fn get_all(&self) -> Vec<LogEntry> {
        self.with_guard(|v| v.logs().cloned().collect())
            .unwrap_or_default()
    }

    pub fn take(self) -> Option<LogCollectorData> {
        let mut guard = self.data.lock().unwrap_or_else(|e| e.into_inner());
        guard.take()
    }

    pub fn make_subscriber(&self) -> impl Subscriber + Send + Sync + 'static {
        tracing_subscriber::registry().with(self.clone())
    }

    pub fn find(&self, mut f: impl FnMut(&LogEntry) -> bool) -> Option<LogEntry> {
        self.with_guard(move |d| d.logs().find(|e| f(e)).cloned())
            .flatten()
    }

    pub fn get(&self, f: impl FnMut(&LogEntry) -> bool) -> LogEntry {
        self.find(f).unwrap()
    }
}

impl<S> Layer<S> for LogCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let mut spans = Vec::new();
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let extensions = span.extensions();
                if let Some(data) = extensions.get::<FieldVisitor>() {
                    let mut fields = data.fields.clone();
                    if let Some(message) = data.message.as_ref() {
                        fields.push(LogField::new("message", message))
                    }

                    spans.push(SpanData {
                        name: span.name().to_string(),
                        fields,
                    });
                }
            }
        }

        let metadata = event.metadata();

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: *metadata.level(),
            target: metadata.target().to_string(),
            message: visitor.message.unwrap_or_default(),
            fields: visitor.fields,
            spans,
        };

        self.push(entry)
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        let mut visitor = FieldVisitor::new();
        attrs.record(&mut visitor);

        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(visitor)
        }
    }
}

struct FieldVisitor {
    message: Option<String>,
    fields: Vec<LogField>,
}

impl FieldVisitor {
    fn new() -> Self {
        FieldVisitor {
            message: None,
            fields: Vec::new(),
        }
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let value = format!("{:?}", value);
        if field.name() == "message" {
            self.message = Some(value);
        } else {
            self.fields.push(LogField {
                name: field.name().to_owned(),
                value,
            });
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push(LogField {
                name: field.name().to_owned(),
                value: value.to_string(),
            });
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::test::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    use tracing::instrument::WithSubscriber;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn non_async_context() {
        let logs = LogCollector::default();
        let subscriber = tracing_subscriber::registry().with(logs.clone());
        tracing::subscriber::with_default(subscriber, || {
            tracing::debug!(b = 11, b = 123, "debug log");
        });
        let entries = logs
            .get_all()
            .into_iter()
            .map(|v| (v.level, v.message, v.fields))
            .collect::<Vec<_>>();
        let expected = Vec::from([(
            Level::DEBUG,
            "debug log".to_owned(),
            Vec::from([LogField::new("b", "11"), LogField::new("b", "123")]),
        )]);
        assert_eq!(entries.as_slice(), expected.as_slice());
    }

    #[tokio::test]
    async fn async_context() {
        let logs = LogCollector::default();
        let subscriber = tracing_subscriber::registry().with(logs.clone());
        async move {
            tracing::info!("1");
            tokio::time::sleep(Duration::from_millis(1)).await;
            tracing::info!("2");
        }
        .with_subscriber(subscriber)
        .await;

        let entries = logs
            .get_all()
            .into_iter()
            .map(|v| (v.level, v.message, v.fields))
            .collect::<Vec<_>>();
        let expected = Vec::from([
            (Level::INFO, "1".to_owned(), Vec::default()),
            (Level::INFO, "2".to_owned(), Vec::default()),
        ]);
        assert_eq!(entries, expected);
    }
}
