//! Contoh pencatatan otomatis lewat `LogsActivity` dan `LogOptions`.

use std::collections::BTreeMap;

use activitylog::support::attribute_change::AttributeChanges;
use activitylog::{
    log_model_event, ActivityEvent, LogOptions, LogsActivity, MemoryStorage, Result, Subject,
};
use serde_json::{json, Value};

struct Post {
    id: u32,
    options: LogOptions,
}

impl Subject for Post {
    fn activity_type(&self) -> String {
        "Post".to_string()
    }

    fn activity_id(&self) -> String {
        self.id.to_string()
    }
}

impl LogsActivity for Post {
    fn activity_log_options(&self) -> LogOptions {
        self.options.clone()
    }
}

fn map(value: Value) -> BTreeMap<String, Value> {
    serde_json::from_value(value).unwrap_or_default()
}

#[tokio::main]
async fn main() -> Result<()> {
    let storage = MemoryStorage::new();
    let post = Post {
        id: 1,
        options: LogOptions::default().use_log_name("posts").log_only_dirty(),
    };

    let changes = AttributeChanges::detect(
        &BTreeMap::new(),
        &map(json!({ "title": "Hello world", "status": "draft" })),
    );

    if let Some(saved) = log_model_event(
        &storage,
        &post,
        ActivityEvent::Created,
        "Post created",
        changes,
        None,
    )
    .await?
    {
        println!("{} -> {}", saved.log_name, saved.description);
    }

    Ok(())
}
