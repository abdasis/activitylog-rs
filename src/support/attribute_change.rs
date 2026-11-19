//! Perubahan atribut (old/new) yang dicatat pada activity.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Nilai lama dan baru per atribut.
///
/// Bentuk JSON mengikuti spatie: `{"old": {...}, "attributes": {...}}`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AttributeChanges {
    /// Nilai sebelum perubahan.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub old: BTreeMap<String, Value>,
    /// Nilai sesudah perubahan.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, Value>,
}

impl AttributeChanges {
    /// Membuat perubahan dari dua peta nilai.
    pub fn new(old: BTreeMap<String, Value>, attributes: BTreeMap<String, Value>) -> Self {
        Self { old, attributes }
    }

    /// Menghitung perubahan dirty saja: atribut yang nilainya berbeda.
    pub fn detect(old: &BTreeMap<String, Value>, attributes: &BTreeMap<String, Value>) -> Self {
        let keys: BTreeSet<&String> = old.keys().chain(attributes.keys()).collect();
        let mut changes = Self::default();

        for key in keys {
            let old_value = old.get(key);
            let new_value = attributes.get(key);
            if old_value == new_value {
                continue;
            }
            if let Some(value) = old_value {
                changes.old.insert(key.clone(), value.clone());
            }
            if let Some(value) = new_value {
                changes.attributes.insert(key.clone(), value.clone());
            }
        }

        changes
    }

    /// `true` bila tidak ada nilai lama maupun baru.
    pub fn is_empty(&self) -> bool {
        self.old.is_empty() && self.attributes.is_empty()
    }

    /// Nilai baru untuk satu atribut.
    pub fn new_value(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }

    /// Nilai lama untuk satu atribut.
    pub fn old_value(&self, key: &str) -> Option<&Value> {
        self.old.get(key)
    }
}
