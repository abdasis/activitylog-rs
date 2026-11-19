//! Contoh mencatat activity manual memakai penyimpanan memori.

use activitylog::models::ActivityQuery;
use activitylog::{activity, ActivityEvent, MemoryStorage, Result, Subject};

struct User {
    id: u32,
}

impl Subject for User {
    fn activity_type(&self) -> String {
        "User".to_string()
    }

    fn activity_id(&self) -> String {
        self.id.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let storage = MemoryStorage::new();
    let user = User { id: 1 };

    activity("auth")
        .performed_on(&user)
        .event(ActivityEvent::Created)
        .with_property("ip", "127.0.0.1")
        .log("User logged in")
        .save(&storage)
        .await?;

    let rows = ActivityQuery::new().in_log("auth").get(&storage).await?;
    for row in rows {
        let event = row
            .event
            .as_ref()
            .map(|value| value.to_string())
            .unwrap_or_default();
        println!("{event}: {}", row.description);
    }

    Ok(())
}
