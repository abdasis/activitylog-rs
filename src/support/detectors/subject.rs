//! Deteksi pasangan (tipe, id) subjek.

use crate::contracts::Subject;

/// Mengambil tipe dan id dari sebuah subjek.
pub fn detect(subject: &impl Subject) -> (String, String) {
    (subject.activity_type(), subject.activity_id())
}
