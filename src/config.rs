//! Konfigurasi default crate.

use crate::error::Result;
use crate::models::delete_older_than;
use crate::storage::ActivityStorage;

/// Nama log yang dipakai bila logger tidak menyebutkan nama.
pub const DEFAULT_LOG_NAME: &str = "default";

/// Umur record default sebelum dibersihkan, dalam hari.
pub const DELETE_RECORDS_OLDER_THAN_DAYS: i64 = 365;

/// Konfigurasi sederhana, setara `config/activitylog.php` milik spatie.
#[derive(Debug, Clone)]
pub struct ActivityConfig {
    /// Nama log default.
    pub default_log_name: String,
    /// Bersihkan record lebih lama dari N hari. `None` berarti tidak dibersihkan.
    pub delete_records_older_than_days: Option<i64>,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            default_log_name: DEFAULT_LOG_NAME.to_string(),
            delete_records_older_than_days: Some(DELETE_RECORDS_OLDER_THAN_DAYS),
        }
    }
}

/// Menjalankan pembersihan sesuai konfigurasi dan mengembalikan jumlah record
/// yang dihapus.
pub async fn cleanup<S: ActivityStorage>(storage: &S, config: &ActivityConfig) -> Result<u64> {
    match config.delete_records_older_than_days {
        Some(days) => delete_older_than(storage, days).await,
        None => Ok(0),
    }
}
