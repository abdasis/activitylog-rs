//! Implementasi storage memakai sqlx dengan query runtime (tanpa makro
//! compile-time, tanpa `DATABASE_URL`).

use chrono::{DateTime, Utc};

use super::{ActivityFilter, ActivityStorage};
use crate::activity::{format_timestamp, Activity};
use crate::error::{Error, Result};

/// Pool sesuai driver yang aktif. Urutan prioritas: sqlite, postgres, mysql.
#[cfg(feature = "sqlite")]
pub type Pool = sqlx::SqlitePool;

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub type Pool = sqlx::PgPool;

#[cfg(all(feature = "mysql", not(feature = "sqlite"), not(feature = "postgres")))]
pub type Pool = sqlx::MySqlPool;

const COLUMNS: &str = "id, log_name, description, subject_type, subject_id, \
causer_type, causer_id, properties, attribute_changes, event, batch_uuid, \
created_at, updated_at";

/// Storage berbasis sqlx.
pub struct SqlxStorage {
    pool: Pool,
}

impl SqlxStorage {
    /// Membuat storage dari pool yang sudah terhubung.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Pool yang dipakai storage ini.
    pub fn pool(&self) -> &Pool {
        &self.pool
    }
}

fn db_error(error: sqlx::Error) -> Error {
    Error::Database(error.to_string())
}

/// Placeholder mengikuti driver aktif: `$n` untuk postgres, `?` untuk sqlite
/// dan mysql.
fn placeholder(index: usize) -> String {
    #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
    {
        format!("${index}")
    }
    #[cfg(not(all(feature = "postgres", not(feature = "sqlite"))))]
    {
        let _ = index;
        "?".to_string()
    }
}

fn add_eq(clauses: &mut Vec<String>, binds: &mut Vec<String>, column: &str, value: &str) {
    binds.push(value.to_string());
    clauses.push(format!("{column} = {}", placeholder(binds.len())));
}

fn where_clause(filter: &ActivityFilter) -> (String, Vec<String>) {
    let mut clauses = Vec::new();
    let mut binds = Vec::new();

    if let Some(value) = &filter.log_name {
        add_eq(&mut clauses, &mut binds, "log_name", value);
    }
    if let Some((kind, id)) = &filter.subject {
        add_eq(&mut clauses, &mut binds, "subject_type", kind);
        add_eq(&mut clauses, &mut binds, "subject_id", id);
    }
    if let Some((kind, id)) = &filter.causer {
        add_eq(&mut clauses, &mut binds, "causer_type", kind);
        add_eq(&mut clauses, &mut binds, "causer_id", id);
    }
    if let Some(event) = &filter.event {
        add_eq(&mut clauses, &mut binds, "event", event.as_str());
    }
    if let Some(value) = &filter.batch_uuid {
        add_eq(&mut clauses, &mut binds, "batch_uuid", value);
    }
    if let Some(from) = filter.from {
        binds.push(format_timestamp(from));
        clauses.push(format!("created_at >= {}", placeholder(binds.len())));
    }
    if let Some(to) = filter.to {
        binds.push(format_timestamp(to));
        clauses.push(format!("created_at <= {}", placeholder(binds.len())));
    }

    if clauses.is_empty() {
        (String::new(), binds)
    } else {
        (format!(" WHERE {}", clauses.join(" AND ")), binds)
    }
}

fn count_sql(where_sql: &str) -> String {
    #[cfg(all(feature = "mysql", not(feature = "sqlite"), not(feature = "postgres")))]
    {
        format!("SELECT CAST(COUNT(*) AS SIGNED) FROM activity_log{where_sql}")
    }
    #[cfg(not(all(feature = "mysql", not(feature = "sqlite"), not(feature = "postgres"))))]
    {
        format!("SELECT COUNT(*) FROM activity_log{where_sql}")
    }
}

#[allow(async_fn_in_trait)]
impl ActivityStorage for SqlxStorage {
    async fn insert(&self, activity: &Activity) -> Result<Activity> {
        let placeholders: Vec<String> = (1..=13).map(placeholder).collect();
        let sql = format!(
            "INSERT INTO activity_log ({COLUMNS}) VALUES ({})",
            placeholders.join(", ")
        );

        let properties = serde_json::to_string(&activity.properties)?;
        let attribute_changes = match &activity.attribute_changes {
            Some(changes) => Some(serde_json::to_string(changes)?),
            None => None,
        };
        let event = activity
            .event
            .as_ref()
            .map(|value| value.as_str().to_string());
        let updated_at = activity.updated_at.map(format_timestamp);

        sqlx::query(&sql)
            .bind(&activity.id)
            .bind(&activity.log_name)
            .bind(&activity.description)
            .bind(&activity.subject_type)
            .bind(&activity.subject_id)
            .bind(&activity.causer_type)
            .bind(&activity.causer_id)
            .bind(&properties)
            .bind(&attribute_changes)
            .bind(&event)
            .bind(&activity.batch_uuid)
            .bind(format_timestamp(activity.created_at))
            .bind(updated_at)
            .execute(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(activity.clone())
    }

    async fn fetch(&self, filter: &ActivityFilter) -> Result<Vec<Activity>> {
        let (where_sql, binds) = where_clause(filter);
        let order = if filter.descending { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT {COLUMNS} FROM activity_log{where_sql} \
             ORDER BY created_at {order}, id {order}"
        );

        let mut query = sqlx::query_as::<_, Activity>(&sql);
        for value in binds {
            query = query.bind(value);
        }

        query.fetch_all(&self.pool).await.map_err(db_error)
    }

    async fn count(&self, filter: &ActivityFilter) -> Result<i64> {
        let (where_sql, binds) = where_clause(filter);
        let sql = count_sql(&where_sql);
        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        for value in binds {
            query = query.bind(value);
        }

        query.fetch_one(&self.pool).await.map_err(db_error)
    }

    async fn delete_before(&self, cutoff: DateTime<Utc>) -> Result<u64> {
        let sql = format!(
            "DELETE FROM activity_log WHERE created_at < {}",
            placeholder(1)
        );
        let result = sqlx::query(&sql)
            .bind(format_timestamp(cutoff))
            .execute(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(result.rows_affected())
    }
}
