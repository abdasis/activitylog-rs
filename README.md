# activitylog-rs

Reimplementasi [spatie/laravel-activitylog](https://github.com/spatie/laravel-activitylog) v4 untuk Rust.
Library crate, async, tanpa proc-macro dan tanpa `unsafe`.

## Fitur

- Record activity: subjek, causer, properti JSON, perubahan atribut (old/new), event, batch UUID.
- Fluent logger: `activity("log").performed_on(..).caused_by(..).event(..).with_property(..).log(..).save(..)`.
- `LogOptions` untuk pencatatan otomatis: `log_all`, `log_only`, `log_except`, `log_only_dirty`,
  `dont_submit_empty_logs`, `use_log_name`, `set_event`.
- Query builder: `in_log`, `for_subject`, `caused_by`, `for_event`, `in_batch`, `between`,
  `latest`/`oldest`, `first`, `count`, `delete_older_than`.
- Storage abstraction `ActivityStorage` dengan implementasi `SqlxStorage` (SQLite/PostgreSQL/MySQL)
  dan `MemoryStorage` (fitur `memory`).
- Semua query sqlx memakai query runtime. Tidak butuh `DATABASE_URL` atau `.sqlx` offline.

## Instalasi

```toml
[dependencies]
activitylog-rs = { path = "../activitylog" }
```

Nama paket `activitylog-rs`, nama library tetap `activitylog` sehingga
`use activitylog::...` dipakai di kode.

Fitur: `sqlite` (default), `postgres`, `mysql`, `memory` (default). Driver pertama yang aktif menang
dengan urutan sqlite, postgres, mysql.

## Pakai cepat

```rust
use activitylog::{activity, ActivityEvent, MemoryStorage, Result, Subject};

struct User { id: u32 }

impl Subject for User {
    fn activity_type(&self) -> String { "User".to_string() }
    fn activity_id(&self) -> String { self.id.to_string() }
}

#[tokio::main]
async fn main() -> Result<()> {
    let storage = MemoryStorage::new();
    activity("auth")
        .performed_on(&User { id: 1 })
        .event(ActivityEvent::Created)
        .with_property("ip", "127.0.0.1")
        .log("User logged in")
        .save(&storage)
        .await?;
    Ok(())
}
```

Contoh lengkap ada di `examples/basic.rs` dan `examples/auto_log.rs`.

## Memakai sqlx

```rust
use sqlx::sqlite::SqlitePoolOptions;
use activitylog::{ActivityStorage, SqlxStorage};

let pool = SqlitePoolOptions::new()
    .max_connections(1)
    .connect("sqlite::memory:")
    .await?;
sqlx::raw_sql(activitylog::migrations::SQLITE_SCHEMA)
    .execute(&pool)
    .await?;
let storage = SqlxStorage::new(pool);
```

Skema juga tersedia sebagai berkas di `migrations/sqlite/`, `migrations/postgres/`, dan
`migrations/mysql/` dengan nama `2019_12_14_000001_create_activity_log_table.sql`, atau lewat konstanta
`activitylog::migrations::SQLITE_SCHEMA`, `POSTGRES_SCHEMA`, dan `MYSQL_SCHEMA`.

## Pencatatan otomatis

```rust
use activitylog::{log_model_event, ActivityEvent, LogOptions, LogsActivity, Subject};
use activitylog::support::attribute_change::AttributeChanges;

struct Post { id: u32, options: LogOptions }

impl Subject for Post {
    fn activity_type(&self) -> String { "Post".to_string() }
    fn activity_id(&self) -> String { self.id.to_string() }
}

impl LogsActivity for Post {
    fn activity_log_options(&self) -> LogOptions { self.options.clone() }
}

let post = Post {
    id: 1,
    options: LogOptions::default().use_log_name("posts").log_only_dirty(),
};
let old_attributes = std::collections::BTreeMap::new();
let new_attributes = std::collections::BTreeMap::from([
    ("title".to_string(), serde_json::json!("Hello")),
]);
let changes = AttributeChanges::detect(&old_attributes, &new_attributes);
let saved = log_model_event(
    &storage,
    &post,
    ActivityEvent::Updated,
    "Post updated",
    changes,
    None,
).await?;
```

`log_model_event` mengembalikan `None` bila `dont_submit_empty_logs` aktif dan tidak ada atribut
yang dicatat.

Batch UUID dapat dibagi dalam satu request dengan `support::detectors::Batch`, lalu diteruskan lewat
`.with_batch_uuid(batch.uuid())`. Hook sebelum penyimpanan memakai `.tap(|pending| { ... })`.

## Pemetaan dari spatie

| spatie | crate ini |
| --- | --- |
| `src/Activity.php` | `src/activity.rs` (`Activity`) |
| `src/ActivityLogger.php` | `src/logger.rs` (`activity()`, `PendingActivity`) |
| `src/LogOptions.php` | `src/log_options.rs` |
| `src/Contracts/Activity.php` | `src/contracts.rs` (`ActivityContract`, `Subject`, `Causer`) |
| `src/Exceptions/*` | `src/error.rs` (`Error`) |
| `src/Models/Activity.php` | `src/models/mod.rs` (`ActivityQuery`) |
| `src/Traits/LogsActivity.php` | `src/traits/logs_activity.rs` |
| `src/Traits/CausesActivity.php` | `src/traits/causes_activity.rs` |
| `src/Support/AttributeChangeDetector` | `src/support/attribute_change.rs` |
| `src/Support/Detectors/*` | `src/support/detectors/*` |
| `config/activitylog.php` | `src/config.rs` (`ActivityConfig`) |
| `database/migrations/*` | `src/migrations.rs` + `migrations/<driver>/*.sql` |

Perbedaan yang disengaja:

- `id` berupa UUID v4 (teks), bukan auto-increment, agar portabel antar driver.
- API async; trait storage memakai async fn in trait bawaan Rust.
- Tidak ada hook global statis; gunakan `.tap()`.
- Tidak ada event model ORM; panggil `log_model_event` dari kode aplikasi.
- Timestamp disimpan sebagai teks RFC3339 UTC berlebar tetap agar portabel dan tetap bisa diurutkan.

## Lisensi

Dirilis dengan lisensi MIT. Lihat berkas [LICENSE](LICENSE).
