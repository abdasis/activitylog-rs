//! Skema tabel per driver, siap ditempel atau dijalankan sendiri oleh user.

/// Skema SQLite.
pub const SQLITE_SCHEMA: &str =
    include_str!("../migrations/sqlite/2019_12_14_000001_create_activity_log_table.sql");

/// Skema PostgreSQL.
pub const POSTGRES_SCHEMA: &str =
    include_str!("../migrations/postgres/2019_12_14_000001_create_activity_log_table.sql");

/// Skema MySQL.
pub const MYSQL_SCHEMA: &str =
    include_str!("../migrations/mysql/2019_12_14_000001_create_activity_log_table.sql");
