//! Deteksi event dari bentuk perubahan atribut.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::enums::ActivityEvent;

/// Menebak event: nilai lama kosong berarti `created`, nilai baru kosong
/// berarti `deleted`, selain itu `updated`.
pub fn detect(
    old: &BTreeMap<String, Value>,
    attributes: &BTreeMap<String, Value>,
) -> ActivityEvent {
    match (old.is_empty(), attributes.is_empty()) {
        (true, false) => ActivityEvent::Created,
        (false, true) => ActivityEvent::Deleted,
        _ => ActivityEvent::Updated,
    }
}
