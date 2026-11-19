//! Query builder untuk record activity.

use chrono::{DateTime, Duration, Utc};

use crate::activity::Activity;
use crate::contracts::{Causer, Subject};
use crate::enums::ActivityEvent;
use crate::error::{Error, Result};
use crate::storage::{ActivityFilter, ActivityStorage};

/// Builder filter untuk query activity.
#[derive(Debug, Clone, Default)]
pub struct ActivityQuery {
    filter: ActivityFilter,
}

impl ActivityQuery {
    /// Query tanpa filter, urut dari terbaru.
    pub fn new() -> Self {
        Self::default()
    }

    /// Batasi ke satu nama log.
    pub fn in_log(mut self, log_name: impl Into<String>) -> Self {
        self.filter.log_name = Some(log_name.into());
        self
    }

    /// Batasi ke satu subjek.
    pub fn for_subject<S: Subject + ?Sized>(mut self, subject: &S) -> Self {
        self.filter.subject = Some((subject.activity_type(), subject.activity_id()));
        self
    }

    /// Batasi ke satu causer.
    pub fn caused_by<C: Causer + ?Sized>(mut self, causer: &C) -> Self {
        self.filter.causer = Some((causer.activity_type(), causer.activity_id()));
        self
    }

    /// Batasi ke satu event.
    pub fn for_event(mut self, event: impl Into<ActivityEvent>) -> Self {
        self.filter.event = Some(event.into());
        self
    }

    /// Batasi ke satu batch UUID.
    pub fn in_batch(mut self, batch_uuid: impl Into<String>) -> Self {
        self.filter.batch_uuid = Some(batch_uuid.into());
        self
    }

    /// Batasi rentang `created_at` (inklusif).
    pub fn between(mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        self.filter.from = Some(from);
        self.filter.to = Some(to);
        self
    }

    /// Urutkan dari yang terbaru.
    pub fn latest(mut self) -> Self {
        self.filter.descending = true;
        self
    }

    /// Urutkan dari yang terlama.
    pub fn oldest(mut self) -> Self {
        self.filter.descending = false;
        self
    }

    /// Filter yang sudah dibangun.
    pub fn filter(&self) -> &ActivityFilter {
        &self.filter
    }

    /// Ambil semua activity yang cocok.
    pub async fn get<S: ActivityStorage>(self, storage: &S) -> Result<Vec<Activity>> {
        storage.fetch(&self.filter).await
    }

    /// Ambil activity pertama sesuai urutan.
    pub async fn first<S: ActivityStorage>(self, storage: &S) -> Result<Option<Activity>> {
        let rows = storage.fetch(&self.filter).await?;
        Ok(rows.into_iter().next())
    }

    /// Hitung activity yang cocok.
    pub async fn count<S: ActivityStorage>(self, storage: &S) -> Result<i64> {
        storage.count(&self.filter).await
    }
}

/// Activity terbaru, bila ada.
pub async fn latest<S: ActivityStorage>(storage: &S) -> Result<Option<Activity>> {
    ActivityQuery::new().latest().first(storage).await
}

/// Hapus activity yang lebih lama dari `days` hari.
pub async fn delete_older_than<S: ActivityStorage>(storage: &S, days: i64) -> Result<u64> {
    let duration = Duration::try_days(days)
        .ok_or_else(|| Error::InvalidData(format!("rentang hari tidak valid: {days}")))?;
    storage.delete_before(Utc::now() - duration).await
}
