//! Trait `CausesActivity`: akses activity milik seorang causer.

use crate::activity::Activity;
use crate::contracts::Causer;
use crate::error::Result;
use crate::models::ActivityQuery;
use crate::storage::ActivityStorage;

/// Causer yang bisa menanyakan activity miliknya.
pub trait CausesActivity: Causer {
    /// Query activity yang disebabkan oleh `self`.
    fn activity_query(&self) -> ActivityQuery {
        ActivityQuery::new().caused_by(self)
    }

    /// Ambil semua activity yang disebabkan oleh `self`.
    async fn activities<S: ActivityStorage>(&self, storage: &S) -> Result<Vec<Activity>> {
        self.activity_query().get(storage).await
    }
}
