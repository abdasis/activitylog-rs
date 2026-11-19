//! Opsi pencatatan otomatis saat model berubah.

use std::collections::{BTreeMap, BTreeSet};

use crate::enums::ActivityEvent;
use crate::support::attribute_change::AttributeChanges;

/// Pengaturan atribut yang dicatat saat activity dibuat otomatis.
#[derive(Debug, Clone)]
pub struct LogOptions {
    /// Nama log yang dipakai. `None` berarti nama default.
    pub log_name: Option<String>,
    /// Catat semua atribut.
    pub log_all: bool,
    /// Bila tidak kosong, hanya atribut ini yang dicatat.
    pub log_only: Vec<String>,
    /// Atribut yang tidak pernah dicatat.
    pub log_except: Vec<String>,
    /// Hanya catat atribut yang nilainya berubah.
    pub log_only_dirty: bool,
    /// Jangan simpan activity bila tidak ada atribut yang dicatat.
    pub dont_submit_empty_logs: bool,
    /// Event default bila pemanggil tidak menentukan event.
    pub event: Option<ActivityEvent>,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            log_name: None,
            log_all: true,
            log_only: Vec::new(),
            log_except: Vec::new(),
            log_only_dirty: false,
            dont_submit_empty_logs: true,
            event: None,
        }
    }
}

impl LogOptions {
    /// Memakai nama log tertentu.
    pub fn use_log_name(mut self, log_name: impl Into<String>) -> Self {
        self.log_name = Some(log_name.into());
        self
    }

    /// Menetapkan event default.
    pub fn set_event(mut self, event: impl Into<ActivityEvent>) -> Self {
        self.event = Some(event.into());
        self
    }

    /// Menetapkan daftar atribut yang diizinkan.
    pub fn log_only(mut self, attributes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.log_only = attributes.into_iter().map(Into::into).collect();
        self
    }

    /// Menetapkan daftar atribut yang dikecualikan.
    pub fn log_except(mut self, attributes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.log_except = attributes.into_iter().map(Into::into).collect();
        self
    }

    /// Hanya mencatat atribut yang berubah.
    pub fn log_only_dirty(mut self) -> Self {
        self.log_only_dirty = true;
        self
    }

    /// Tetap menyimpan activity walau tidak ada atribut yang dicatat.
    pub fn submit_empty_logs(mut self) -> Self {
        self.dont_submit_empty_logs = false;
        self
    }

    /// Menyaring perubahan atribut sesuai opsi. `None` berarti activity tidak
    /// perlu disimpan.
    pub fn filter_changes(&self, changes: &AttributeChanges) -> Option<AttributeChanges> {
        let keys: BTreeSet<&String> = changes
            .old
            .keys()
            .chain(changes.attributes.keys())
            .collect();
        let mut old = BTreeMap::new();
        let mut attributes = BTreeMap::new();

        for key in keys {
            if !self.log_only.is_empty() && !self.log_only.iter().any(|allowed| allowed == key) {
                continue;
            }
            if self.log_except.iter().any(|excluded| excluded == key) {
                continue;
            }

            let old_value = changes.old.get(key);
            let new_value = changes.attributes.get(key);
            if self.log_only_dirty && old_value == new_value {
                continue;
            }
            if let Some(value) = old_value {
                old.insert(key.clone(), value.clone());
            }
            if let Some(value) = new_value {
                attributes.insert(key.clone(), value.clone());
            }
        }

        if !self.log_all && self.log_only.is_empty() {
            old.clear();
            attributes.clear();
        }

        if old.is_empty() && attributes.is_empty() && self.dont_submit_empty_logs {
            return None;
        }

        Some(AttributeChanges::new(old, attributes))
    }
}
