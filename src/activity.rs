//! Record activity yang disimpan dan dibaca kembali.

use std::collections::BTreeMap;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use chrono::SecondsFormat;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contracts::ActivityContract;
use crate::enums::ActivityEvent;
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::error::{Error, Result};
use crate::support::attribute_change::AttributeChanges;

/// Satu baris pada tabel activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    /// Identitas unik (UUID v4 dalam bentuk teks).
    pub id: String,
    /// Nama log, misalnya `auth` atau `posts`.
    pub log_name: String,
    /// Deskripsi naratif untuk pembaca.
    pub description: String,
    /// Tipe morfologi subjek.
    pub subject_type: Option<String>,
    /// Kunci subjek.
    pub subject_id: Option<String>,
    /// Tipe morfologi causer.
    pub causer_type: Option<String>,
    /// Kunci causer.
    pub causer_id: Option<String>,
    /// Properti bebas.
    pub properties: BTreeMap<String, Value>,
    /// Nilai lama dan baru per atribut.
    pub attribute_changes: Option<AttributeChanges>,
    /// Event activity.
    pub event: Option<ActivityEvent>,
    /// UUID yang dibagi dalam satu batch/request.
    pub batch_uuid: Option<String>,
    /// Waktu dibuat (UTC).
    pub created_at: DateTime<Utc>,
    /// Waktu terakhir diubah (UTC), bila ada.
    pub updated_at: Option<DateTime<Utc>>,
}

impl Activity {
    /// Membuat activity baru dengan id UUID dan waktu sekarang.
    pub fn new(log_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            log_name: log_name.into(),
            description: description.into(),
            subject_type: None,
            subject_id: None,
            causer_type: None,
            causer_id: None,
            properties: BTreeMap::new(),
            attribute_changes: None,
            event: None,
            batch_uuid: None,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Mengambil satu properti.
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }
}

impl ActivityContract for Activity {
    fn id(&self) -> &str {
        &self.id
    }

    fn log_name(&self) -> &str {
        &self.log_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn subject(&self) -> Option<(&str, &str)> {
        match (&self.subject_type, &self.subject_id) {
            (Some(kind), Some(id)) => Some((kind, id)),
            _ => None,
        }
    }

    fn causer(&self) -> Option<(&str, &str)> {
        match (&self.causer_type, &self.causer_id) {
            (Some(kind), Some(id)) => Some((kind, id)),
            _ => None,
        }
    }

    fn event(&self) -> Option<&ActivityEvent> {
        self.event.as_ref()
    }

    fn batch_uuid(&self) -> Option<&str> {
        self.batch_uuid.as_deref()
    }

    fn changes(&self) -> Option<&AttributeChanges> {
        self.attribute_changes.as_ref()
    }

    fn properties(&self) -> &BTreeMap<String, Value> {
        &self.properties
    }
}

/// Format waktu yang dipakai di semua driver: UTC, lebar tetap, urut secara
/// leksikografis.
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub(crate) fn format_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, true)
}

/// Membaca kembali waktu yang disimpan oleh [`format_timestamp`].
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub(crate) fn parse_timestamp(raw: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| Error::InvalidData(format!("timestamp tidak valid: {error}")))
}

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
impl<'r> sqlx::FromRow<'r, crate::storage::DbRow> for Activity {
    fn from_row(row: &'r crate::storage::DbRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::Row;

        fn decode(error: impl std::error::Error + Send + Sync + 'static) -> sqlx::Error {
            sqlx::Error::Decode(Box::new(error))
        }

        let raw_properties: String = row.try_get("properties")?;
        let raw_changes: Option<String> = row.try_get("attribute_changes")?;
        let raw_event: Option<String> = row.try_get("event")?;
        let raw_created_at: String = row.try_get("created_at")?;
        let raw_updated_at: Option<String> = row.try_get("updated_at")?;

        Ok(Self {
            id: row.try_get("id")?,
            log_name: row.try_get("log_name")?,
            description: row.try_get("description")?,
            subject_type: row.try_get("subject_type")?,
            subject_id: row.try_get("subject_id")?,
            causer_type: row.try_get("causer_type")?,
            causer_id: row.try_get("causer_id")?,
            properties: serde_json::from_str(&raw_properties).map_err(decode)?,
            attribute_changes: match raw_changes {
                Some(raw) => Some(serde_json::from_str(&raw).map_err(decode)?),
                None => None,
            },
            event: raw_event.map(|value| ActivityEvent::parse(&value)),
            batch_uuid: row.try_get("batch_uuid")?,
            created_at: parse_timestamp(&raw_created_at).map_err(decode)?,
            updated_at: match raw_updated_at {
                Some(raw) => Some(parse_timestamp(&raw).map_err(decode)?),
                None => None,
            },
        })
    }
}
