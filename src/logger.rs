//! Fluent logger: `activity("auth").performed_on(&user).log("...").save(...)`.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::activity::Activity;
use crate::config;
use crate::contracts::{Causer, Subject};
use crate::enums::ActivityEvent;
use crate::error::{Error, Result};
use crate::storage::ActivityStorage;
use crate::support::attribute_change::AttributeChanges;

/// Activity yang sedang disusun dan belum disimpan.
#[derive(Debug, Clone, Default)]
pub struct PendingActivity {
    log_name: String,
    description: Option<String>,
    subject: Option<(String, String)>,
    causer: Option<(String, String)>,
    event: Option<ActivityEvent>,
    properties: BTreeMap<String, Value>,
    attribute_changes: Option<AttributeChanges>,
    batch_uuid: Option<String>,
}

/// Memulai activity pada nama log tertentu.
pub fn activity(log_name: impl Into<String>) -> PendingActivity {
    PendingActivity {
        log_name: log_name.into(),
        ..PendingActivity::default()
    }
}

impl PendingActivity {
    /// Menetapkan subjek activity.
    pub fn performed_on<S: Subject + ?Sized>(mut self, subject: &S) -> Self {
        self.subject = Some((subject.activity_type(), subject.activity_id()));
        self
    }

    /// Menetapkan pelaku activity.
    pub fn caused_by<C: Causer + ?Sized>(mut self, causer: &C) -> Self {
        self.causer = Some((causer.activity_type(), causer.activity_id()));
        self
    }

    /// Menetapkan event.
    pub fn event(mut self, event: impl Into<ActivityEvent>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Menambah satu properti.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Menambah beberapa properti sekaligus.
    pub fn with_properties<K, V, I>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        for (key, value) in properties {
            self.properties.insert(key.into(), value.into());
        }
        self
    }

    /// Menetapkan perubahan atribut.
    pub fn with_attribute_changes(mut self, changes: AttributeChanges) -> Self {
        self.attribute_changes = Some(changes);
        self
    }

    /// Menetapkan batch UUID.
    pub fn with_batch_uuid(mut self, batch_uuid: impl Into<String>) -> Self {
        self.batch_uuid = Some(batch_uuid.into());
        self
    }

    /// Hook sebelum penyimpanan, setara `tap` milik spatie.
    pub fn tap(mut self, hook: impl FnOnce(&mut Self)) -> Self {
        hook(&mut self);
        self
    }

    /// Menetapkan deskripsi. Harus dipanggil sebelum [`Self::save`].
    pub fn log(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mengubah nama log dari dalam hook.
    pub fn set_log_name(&mut self, log_name: impl Into<String>) {
        self.log_name = log_name.into();
    }

    /// Mengubah deskripsi dari dalam hook.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Mengubah event dari dalam hook.
    pub fn set_event(&mut self, event: impl Into<ActivityEvent>) {
        self.event = Some(event.into());
    }

    /// Mengubah satu properti dari dalam hook.
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.properties.insert(key.into(), value.into());
    }

    /// Mengubah batch UUID dari dalam hook.
    pub fn set_batch_uuid(&mut self, batch_uuid: impl Into<String>) {
        self.batch_uuid = Some(batch_uuid.into());
    }

    /// Mengubah perubahan atribut dari dalam hook.
    pub fn set_attribute_changes(&mut self, changes: AttributeChanges) {
        self.attribute_changes = Some(changes);
    }

    /// Menyimpan activity ke storage dan mengembalikan record tersimpan.
    pub async fn save<S: ActivityStorage>(self, storage: &S) -> Result<Activity> {
        let description = self.description.ok_or_else(|| {
            Error::InvalidData("deskripsi activity wajib diisi lewat .log(...)".to_string())
        })?;
        let log_name = if self.log_name.is_empty() {
            config::DEFAULT_LOG_NAME.to_string()
        } else {
            self.log_name
        };

        let mut record = Activity::new(log_name, description);
        if let Some((kind, id)) = self.subject {
            record.subject_type = Some(kind);
            record.subject_id = Some(id);
        }
        if let Some((kind, id)) = self.causer {
            record.causer_type = Some(kind);
            record.causer_id = Some(id);
        }
        record.event = self.event;
        record.properties = self.properties;
        record.attribute_changes = self.attribute_changes;
        record.batch_uuid = self.batch_uuid;

        storage.insert(&record).await
    }
}
