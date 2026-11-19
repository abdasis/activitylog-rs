//! Storage in-memory untuk test dan paritas perilaku.

use std::sync::RwLock;

use chrono::{DateTime, Utc};

use super::{ActivityFilter, ActivityStorage};
use crate::activity::Activity;
use crate::error::{Error, Result};

/// Storage sederhana yang menyimpan semua activity di memori.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    rows: RwLock<Vec<Activity>>,
}

impl MemoryStorage {
    /// Membuat storage kosong.
    pub fn new() -> Self {
        Self::default()
    }

    fn poisoned() -> Error {
        Error::InvalidData("memory storage lock poisoned".to_string())
    }
}

#[allow(async_fn_in_trait)]
impl ActivityStorage for MemoryStorage {
    async fn insert(&self, activity: &Activity) -> Result<Activity> {
        let mut rows = self.rows.write().map_err(|_| Self::poisoned())?;
        rows.push(activity.clone());
        Ok(activity.clone())
    }

    async fn fetch(&self, filter: &ActivityFilter) -> Result<Vec<Activity>> {
        let rows = self.rows.read().map_err(|_| Self::poisoned())?;
        let mut result: Vec<Activity> = rows
            .iter()
            .filter(|activity| filter.matches(activity))
            .cloned()
            .collect();

        result.sort_by(|left, right| {
            let order = left
                .created_at
                .cmp(&right.created_at)
                .then_with(|| left.id.cmp(&right.id));
            if filter.descending {
                order.reverse()
            } else {
                order
            }
        });

        Ok(result)
    }

    async fn count(&self, filter: &ActivityFilter) -> Result<i64> {
        let rows = self.rows.read().map_err(|_| Self::poisoned())?;
        Ok(rows
            .iter()
            .filter(|activity| filter.matches(activity))
            .count() as i64)
    }

    async fn delete_before(&self, cutoff: DateTime<Utc>) -> Result<u64> {
        let mut rows = self.rows.write().map_err(|_| Self::poisoned())?;
        let before = rows.len();
        rows.retain(|activity| activity.created_at >= cutoff);
        Ok((before - rows.len()) as u64)
    }
}
