use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;
use time::OffsetDateTime;
use tracing::field::{Field, Visit};
use tracing::Event;

#[derive(Debug, Serialize, Clone, Deserialize, PartialOrd, PartialEq, Ord, Eq, Default)]
#[repr(usize)]
pub(super) enum TracerLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl TracerLevel {
    pub(super) fn to_color32(&self, ui: &egui::Ui) -> egui::Color32 {
        match self {
            TracerLevel::Error => ui.visuals().error_fg_color,
            TracerLevel::Warn => ui.visuals().warn_fg_color,
            TracerLevel::Info => ui.visuals().text_color(),
            TracerLevel::Debug => ui.visuals().weak_text_color(),
            TracerLevel::Trace => ui.visuals().weak_text_color(),
        }
    }
}

impl std::fmt::Display for TracerLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracerLevel::Error => write!(f, "ERROR"),
            TracerLevel::Warn => write!(f, "WARN"),
            TracerLevel::Info => write!(f, "INFO"),
            TracerLevel::Debug => write!(f, "DEBUG"),
            TracerLevel::Trace => write!(f, "TRACE"),
        }
    }
}

impl From<tracing::Level> for TracerLevel {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::ERROR => TracerLevel::Error,
            tracing::Level::WARN => TracerLevel::Warn,
            tracing::Level::INFO => TracerLevel::Info,
            tracing::Level::DEBUG => TracerLevel::Debug,
            tracing::Level::TRACE => TracerLevel::Trace,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct LogsState {
    pub(super) level_filter: BTreeMap<TracerLevel, bool>,
}

impl Default for LogsState {
    fn default() -> Self {
        LogsState {
            level_filter: BTreeMap::from([
                (TracerLevel::Error, true),
                (TracerLevel::Warn, true),
                (TracerLevel::Info, true),
                (TracerLevel::Debug, false),
            ]),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct CollectedEvent {
    pub(super) target: String,
    pub(super) level: TracerLevel,
    pub(super) fields: BTreeMap<String, String>,
    pub(super) time: OffsetDateTime,
}

impl CollectedEvent {
    pub(super) fn new(event: &Event) -> Self {
        let mut fields = BTreeMap::new();
        event.record(&mut FieldVisitor(&mut fields));
        let level: TracerLevel = (*event.metadata().level()).into();
        CollectedEvent {
            level,
            time: OffsetDateTime::now_local().unwrap(),
            target: event.metadata().target().to_owned(),
            fields,
        }
    }
}

struct FieldVisitor<'a>(&'a mut BTreeMap<String, String>);

impl<'a> Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}
