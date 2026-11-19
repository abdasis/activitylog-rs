//! Reimplementasi `spatie/laravel-activitylog` v4 untuk Rust.
//!
//! Crate ini menyimpan activity (subjek, causer, properti, perubahan atribut,
//! event, batch UUID) lewat trait [`ActivityStorage`]. Implementasi tersedia
//! untuk sqlx (SQLite/PostgreSQL/MySQL) dan penyimpanan memori.
//!
//! Alur dasar:
//!
//! ```text
//! activity("auth")
//!     .performed_on(&user)
//!     .caused_by(&admin)
//!     .event(ActivityEvent::Created)
//!     .with_property("ip", "127.0.0.1")
//!     .log("User logged in")
//!     .save(&storage)
//!     .await?
//! ```
//!
//! Skema tabel tersedia di [`migrations`] dan berkas `migrations/<driver>/`.
//!
//! Lisensi MIT.

// Native async fn in trait (Rust 1.75+) dipakai; trait storage tidak pernah
// dijadikan objek `dyn`, jadi tidak perlu crate `async-trait`.
#![allow(async_fn_in_trait)]

pub mod activity;
pub mod config;
pub mod contracts;
pub mod enums;
pub mod error;
pub mod log_options;
pub mod logger;
pub mod migrations;
pub mod models;
pub mod storage;
pub mod support;
pub mod traits;

pub use activity::Activity;
pub use config::ActivityConfig;
pub use contracts::{ActivityContract, Causer, Subject};
pub use enums::ActivityEvent;
pub use error::{Error, Result};
pub use log_options::LogOptions;
pub use logger::{activity, PendingActivity};
pub use models::ActivityQuery;
pub use storage::{ActivityFilter, ActivityStorage};
pub use traits::{log_model_event, CausesActivity, LogsActivity};

#[cfg(feature = "memory")]
pub use storage::MemoryStorage;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub use storage::{Pool, SqlxStorage};
