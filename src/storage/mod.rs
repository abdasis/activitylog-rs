//! Abstraksi penyimpanan activity.

use chrono::{DateTime, Utc};

use crate::activity::Activity;
use crate::enums::ActivityEvent;
use crate::error::Result;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
mod sqlx_storage;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub use sqlx_storage::{Pool, SqlxStorage};

#[cfg(feature = "memory")]
mod memory;

#[cfg(feature = "memory")]
pub use memory::MemoryStorage;

/// Baris database sesuai driver yang aktif. Driver pertama yang aktif menang:
/// sqlite, lalu postgres, lalu mysql.
#[cfg(feature = "sqlite")]
pub(crate) type DbRow = sqlx::sqlite::SqliteRow;

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub(crate) type DbRow = sqlx::postgres::PgRow;

#[cfg(all(feature = "mysql", not(feature = "sqlite"), not(feature = "postgres")))]
pub(crate) type DbRow = sqlx::mysql::MySqlRow;

/// Filter query yang dipakai bersama oleh semua implementasi storage.
#[derive(Debug, Clone)]
pub struct ActivityFilter {
    /// Batasi ke satu nama log.
    pub log_name: Option<String>,
    /// Batasi ke satu subjek `(tipe, id)`.
    pub subject: Option<(String, String)>,
    /// Batasi ke satu causer `(tipe, id)`.
    pub causer: Option<(String, String)>,
    /// Batasi ke satu event.
    pub event: Option<ActivityEvent>,
    /// Batasi ke satu batch UUID.
    pub batch_uuid: Option<String>,
    /// Batas bawah `created_at` (inklusif).
    pub from: Option<DateTime<Utc>>,
    /// Batas atas `created_at` (inklusif).
    pub to: Option<DateTime<Utc>>,
    /// Urutkan dari terbaru bila `true`.
    pub descending: bool,
}

impl Default for ActivityFilter {
    fn default() -> Self {
        Self {
            log_name: None,
            subject: None,
            causer: None,
            event: None,
            batch_uuid: None,
            from: None,
            to: None,
            descending: true,
        }
    }
}

impl ActivityFilter {
    /// Mengecek apakah satu activity lolos filter.
    pub fn matches(&self, activity: &Activity) -> bool {
        if let Some(log_name) = &self.log_name {
            if &activity.log_name != log_name {
                return false;
            }
        }
        if let Some((kind, id)) = &self.subject {
            if activity.subject_type.as_deref() != Some(kind)
                || activity.subject_id.as_deref() != Some(id)
            {
                return false;
            }
        }
        if let Some((kind, id)) = &self.causer {
            if activity.causer_type.as_deref() != Some(kind)
                || activity.causer_id.as_deref() != Some(id)
            {
                return false;
            }
        }
        if let Some(event) = &self.event {
            if activity.event.as_ref() != Some(event) {
                return false;
            }
        }
        if let Some(batch_uuid) = &self.batch_uuid {
            if activity.batch_uuid.as_deref() != Some(batch_uuid) {
                return false;
            }
        }
        if let Some(from) = self.from {
            if activity.created_at < from {
                return false;
            }
        }
        if let Some(to) = self.to {
            if activity.created_at > to {
                return false;
            }
        }
        true
    }
}

/// Penyimpanan yang bisa dipakai crate ini.
///
/// Memakai async fn in trait bawaan Rust; trait ini tidak dipakai sebagai
/// objek `dyn`, jadi tidak perlu crate `async-trait`.
#[allow(async_fn_in_trait)]
pub trait ActivityStorage: Send + Sync {
    /// Menyimpan activity dan mengembalikan record yang tersimpan.
    async fn insert(&self, activity: &Activity) -> Result<Activity>;

    /// Mengambil semua activity yang cocok dengan filter.
    async fn fetch(&self, filter: &ActivityFilter) -> Result<Vec<Activity>>;

    /// Menghitung activity yang cocok dengan filter.
    async fn count(&self, filter: &ActivityFilter) -> Result<i64>;

    /// Menghapus activity yang dibuat sebelum `cutoff` dan mengembalikan jumlahnya.
    async fn delete_before(&self, cutoff: DateTime<Utc>) -> Result<u64>;
}
