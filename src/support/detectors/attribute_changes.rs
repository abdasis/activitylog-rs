//! Deteksi perubahan atribut dirty.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::support::attribute_change::AttributeChanges;

/// Hanya menyimpan atribut yang nilainya berubah antara `old` dan `attributes`.
pub fn detect(
    old: &BTreeMap<String, Value>,
    attributes: &BTreeMap<String, Value>,
) -> AttributeChanges {
    AttributeChanges::detect(old, attributes)
}
