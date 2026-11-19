//! Trait `LogsActivity`: pencatatan otomatis saat model berubah.

use crate::activity::Activity;
use crate::config;
use crate::contracts::{Causer, Subject};
use crate::enums::ActivityEvent;
use crate::error::Result;
use crate::log_options::LogOptions;
use crate::logger::activity;
use crate::storage::ActivityStorage;
use crate::support::attribute_change::AttributeChanges;

/// Model yang bisa mencatat activity secara otomatis.
pub trait LogsActivity: Subject {
    /// Opsi atribut yang dicatat untuk event apa pun.
    fn activity_log_options(&self) -> LogOptions;
}

/// Mencatat satu event model dengan opsi milik model.
///
/// Mengembalikan `None` bila `dont_submit_empty_logs` aktif dan tidak ada
/// atribut yang dicatat.
pub async fn log_model_event<S, M>(
    storage: &S,
    model: &M,
    event: ActivityEvent,
    description: impl Into<String>,
    changes: AttributeChanges,
    causer: Option<&dyn Causer>,
) -> Result<Option<Activity>>
where
    S: ActivityStorage,
    M: LogsActivity,
{
    let options = model.activity_log_options();
    let Some(changes) = options.filter_changes(&changes) else {
        return Ok(None);
    };

    let event = options.event.clone().unwrap_or(event);
    let log_name = options
        .log_name
        .clone()
        .unwrap_or_else(|| config::DEFAULT_LOG_NAME.to_string());

    let mut pending = activity(log_name)
        .performed_on(model)
        .event(event)
        .log(description)
        .with_attribute_changes(changes);

    if let Some(causer) = causer {
        pending = pending.caused_by(causer);
    }

    pending.save(storage).await.map(Some)
}
