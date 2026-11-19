//! Trait kontrak: subjek, causer, dan bentuk activity.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::enums::ActivityEvent;
use crate::support::attribute_change::AttributeChanges;

/// Model yang bisa menjadi subjek activity.
pub trait Subject {
    /// Tipe morfologi subjek, misalnya `User` atau `Post`.
    fn activity_type(&self) -> String;

    /// Kunci subjek dalam bentuk teks.
    fn activity_id(&self) -> String;
}

/// Model yang bisa menjadi causer (pelaku) activity.
pub trait Causer {
    /// Tipe morfologi causer, misalnya `User`.
    fn activity_type(&self) -> String;

    /// Kunci causer dalam bentuk teks.
    fn activity_id(&self) -> String;
}

/// Bentuk minimal yang harus dimiliki record activity.
pub trait ActivityContract {
    /// Identitas unik activity.
    fn id(&self) -> &str;
    /// Nama log.
    fn log_name(&self) -> &str;
    /// Deskripsi naratif.
    fn description(&self) -> &str;
    /// Pasangan (tipe, id) subjek.
    fn subject(&self) -> Option<(&str, &str)>;
    /// Pasangan (tipe, id) causer.
    fn causer(&self) -> Option<(&str, &str)>;
    /// Event activity.
    fn event(&self) -> Option<&ActivityEvent>;
    /// UUID batch.
    fn batch_uuid(&self) -> Option<&str>;
    /// Perubahan atribut lama/baru.
    fn changes(&self) -> Option<&AttributeChanges>;
    /// Properti tambahan.
    fn properties(&self) -> &BTreeMap<String, Value>;
}
