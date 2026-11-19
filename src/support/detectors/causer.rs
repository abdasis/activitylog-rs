//! Deteksi pasangan (tipe, id) causer.

use crate::contracts::Causer;

/// Mengambil tipe dan id dari seorang causer.
pub fn detect(causer: &impl Causer) -> (String, String) {
    (causer.activity_type(), causer.activity_id())
}
