//! Error type for the crate.

use thiserror::Error;

/// Semua operasi crate ini mengembalikan tipe hasil ini.
pub type Result<T> = std::result::Result<T, Error>;

/// Kesalahan yang mungkin muncul saat mencatat atau membaca activity.
#[derive(Debug, Error)]
pub enum Error {
    /// Gagal mengubah data ke/dari JSON.
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Driver database mengembalikan kesalahan.
    #[error("database error: {0}")]
    Database(String),

    /// Data tersimpan tidak valid (timestamp rusak, id kosong, dan sejenisnya).
    #[error("invalid data: {0}")]
    InvalidData(String),
}
