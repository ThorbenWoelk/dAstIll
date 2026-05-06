use std::{error::Error, fmt};

use chrono::{SecondsFormat, Utc};
use tracing::{Event, Level, Subscriber, field::Field};
use tracing_subscriber::{
    Layer, filter,
    fmt::{
        FmtContext,
        format::{FormatEvent, FormatFields, Writer},
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct HumanReadableEventFormatter;

const LOGFIRE_AI_TARGET_PREFIXES: &[&str] = &[
    "dastill::services::chat",
    "dastill::services::ollama",
    "dastill::services::search",
    "dastill::services::summarizer",
    "dastill::services::summary_evaluator",
];

pub fn should_send_to_logfire(target: &str, level: &Level) -> bool {
    *level == Level::ERROR
        || LOGFIRE_AI_TARGET_PREFIXES
            .iter()
            .any(|prefix| target.starts_with(prefix))
}

pub fn init_tracing() -> anyhow::Result<Option<logfire::ShutdownGuard>> {
    let is_cloud_run = std::env::var("K_SERVICE").is_ok();
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "dastill=info,tower_http=info".into());

    if std::env::var("LOGFIRE_TOKEN").is_ok() {
        let logfire = logfire::configure()
            .local()
            .with_service_name("dastill-backend")
            .with_service_version(env!("CARGO_PKG_VERSION"))
            .finish()?;
        let guard = logfire.clone().shutdown_guard();

        let registry = tracing_subscriber::registry().with(env_filter).with(
            logfire
                .tracing_layer()
                .with_filter(filter::filter_fn(|metadata| {
                    should_send_to_logfire(metadata.target(), metadata.level())
                })),
        );

        if is_cloud_run {
            registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .flatten_event(true)
                        .with_current_span(false)
                        .with_span_list(false)
                        .with_ansi(false),
                )
                .init();
        } else {
            registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .event_format(HumanReadableEventFormatter)
                        .with_ansi(false),
                )
                .init();
        }

        Ok(Some(guard))
    } else {
        let registry = tracing_subscriber::registry().with(env_filter);

        if is_cloud_run {
            registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .flatten_event(true)
                        .with_current_span(false)
                        .with_span_list(false)
                        .with_ansi(false),
                )
                .init();
        } else {
            registry
                .with(
                    tracing_subscriber::fmt::layer()
                        .event_format(HumanReadableEventFormatter)
                        .with_ansi(false),
                )
                .init();
        }

        Ok(None)
    }
}

impl<S, N> FormatEvent<S, N> for HumanReadableEventFormatter
where
    S: Subscriber + for<'span> LookupSpan<'span>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let mut visitor = HumanReadableEventVisitor::default();
        event.record(&mut visitor);

        write!(
            writer,
            "{timestamp} {:<5} {}:",
            metadata.level(),
            metadata.target()
        )?;

        if let Some(message) = visitor.message() {
            write!(writer, " {message}")?;
        }

        for (name, value) in visitor.fields() {
            write!(writer, " {name}={value}")?;
        }

        writeln!(writer)
    }
}

#[derive(Debug, Default)]
struct HumanReadableEventVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl HumanReadableEventVisitor {
    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    fn record_field(&mut self, field: &Field, value: String) {
        if field.name().starts_with("logfire.") {
            return;
        }

        if field.name() == "message" {
            self.message = Some(strip_wrapping_quotes(value));
            return;
        }

        self.fields.push((field.name().to_string(), value));
    }
}

impl tracing::field::Visit for HumanReadableEventVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.record_field(field, format!("{value:?}"));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_field(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_field(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_field(field, value.to_string());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_field(field, format!("{value:?}"));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn Error + 'static)) {
        self.record_field(field, value.to_string());
    }
}

fn strip_wrapping_quotes(value: String) -> String {
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value[1..value.len() - 1].to_string()
    } else {
        value
    }
}

#[cfg(test)]
#[path = "logging_tests.rs"]
mod logging_tests;
