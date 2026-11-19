#![cfg(all(feature = "sqlite", feature = "memory"))]

use std::collections::BTreeMap;

use activitylog::models::{delete_older_than, ActivityQuery};
use activitylog::support::attribute_change::AttributeChanges;
use activitylog::support::detectors::event as event_detector;
use activitylog::{
    activity, log_model_event, Activity, ActivityEvent, ActivityStorage, Causer, LogOptions,
    LogsActivity, MemoryStorage, SqlxStorage, Subject,
};
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;

async fn sqlite_storage() -> SqlxStorage {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql(activitylog::migrations::SQLITE_SCHEMA)
        .execute(&pool)
        .await
        .unwrap();
    SqlxStorage::new(pool)
}

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

struct Admin;

impl Causer for Admin {
    fn activity_type(&self) -> String {
        "Admin".to_string()
    }

    fn activity_id(&self) -> String {
        "9".to_string()
    }
}

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
    serde_json::from_value(value).unwrap()
}

fn record(log_name: &str, description: &str, created_at: DateTime<Utc>) -> Activity {
    let mut activity = Activity::new(log_name, description);
    activity.created_at = created_at;
    activity
}

#[tokio::test]
async fn manual_log_roundtrip() {
    let storage = sqlite_storage().await;

    let saved = activity("auth")
        .performed_on(&User { id: 7 })
        .caused_by(&Admin)
        .event(ActivityEvent::Created)
        .with_property("ip", "127.0.0.1")
        .with_batch_uuid("batch-1")
        .log("User logged in")
        .save(&storage)
        .await
        .unwrap();

    assert_eq!(saved.log_name, "auth");
    assert_eq!(saved.description, "User logged in");
    assert_eq!(saved.subject_type.as_deref(), Some("User"));
    assert_eq!(saved.subject_id.as_deref(), Some("7"));
    assert_eq!(saved.causer_type.as_deref(), Some("Admin"));
    assert_eq!(saved.causer_id.as_deref(), Some("9"));
    assert_eq!(saved.event, Some(ActivityEvent::Created));
    assert_eq!(saved.batch_uuid.as_deref(), Some("batch-1"));

    let rows = ActivityQuery::new()
        .in_log("auth")
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, saved.id);
    assert_eq!(rows[0].get_property("ip"), Some(&json!("127.0.0.1")));
}

#[tokio::test]
async fn tap_changes_fields_before_insert() {
    let storage = sqlite_storage().await;

    let saved = activity("auth")
        .log("placeholder")
        .tap(|pending| {
            pending.set_log_name("security");
            pending.set_description("Login dari perangkat baru");
            pending.set_event(ActivityEvent::Custom("login".to_string()));
            pending.set_property("device", "phone");
            pending.set_batch_uuid("batch-tap");
        })
        .save(&storage)
        .await
        .unwrap();

    assert_eq!(saved.log_name, "security");
    assert_eq!(saved.description, "Login dari perangkat baru");
    assert_eq!(
        saved.event,
        Some(ActivityEvent::Custom("login".to_string()))
    );
    assert_eq!(saved.get_property("device"), Some(&json!("phone")));
    assert_eq!(saved.batch_uuid.as_deref(), Some("batch-tap"));
}

#[tokio::test]
async fn log_options_filter_attributes() {
    let created_storage = sqlite_storage().await;
    let created_options = LogOptions::default().use_log_name("posts");
    let post = Post {
        id: 1,
        options: created_options,
    };
    let changes = AttributeChanges::detect(
        &BTreeMap::new(),
        &map(json!({ "title": "Hello", "status": "draft" })),
    );
    let saved = log_model_event(
        &created_storage,
        &post,
        ActivityEvent::Created,
        "Post created",
        changes,
        Some(&Admin),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(saved.log_name, "posts");
    assert_eq!(saved.event, Some(ActivityEvent::Created));
    assert_eq!(saved.causer_id.as_deref(), Some("9"));
    let recorded = saved.attribute_changes.unwrap();
    assert!(recorded.old.is_empty());
    assert_eq!(recorded.attributes.len(), 2);

    let updated_storage = sqlite_storage().await;
    let updated_post = Post {
        id: 1,
        options: LogOptions::default().use_log_name("posts").log_only_dirty(),
    };
    let full_changes = AttributeChanges::new(
        map(json!({ "title": "Old", "status": "draft" })),
        map(json!({ "title": "New", "status": "draft" })),
    );
    let saved = log_model_event(
        &updated_storage,
        &updated_post,
        ActivityEvent::Updated,
        "Post updated",
        full_changes,
        None,
    )
    .await
    .unwrap()
    .unwrap();
    let recorded = saved.attribute_changes.unwrap();
    assert_eq!(recorded.attributes.len(), 1);
    assert_eq!(recorded.attributes.get("title"), Some(&json!("New")));
    assert_eq!(recorded.old.get("title"), Some(&json!("Old")));
    assert!(!recorded.attributes.contains_key("status"));

    let all_storage = sqlite_storage().await;
    let all_post = Post {
        id: 1,
        options: LogOptions::default(),
    };
    let full_changes = AttributeChanges::new(
        map(json!({ "title": "Old", "status": "draft" })),
        map(json!({ "title": "New", "status": "draft" })),
    );
    let saved = log_model_event(
        &all_storage,
        &all_post,
        ActivityEvent::Updated,
        "Post updated",
        full_changes,
        None,
    )
    .await
    .unwrap()
    .unwrap();
    let recorded = saved.attribute_changes.unwrap();
    assert_eq!(recorded.attributes.len(), 2);
    assert_eq!(recorded.attributes.get("status"), Some(&json!("draft")));

    let except_storage = sqlite_storage().await;
    let except_post = Post {
        id: 1,
        options: LogOptions::default().log_except(["status"]),
    };
    let full_changes = AttributeChanges::new(
        map(json!({ "title": "Old", "status": "draft" })),
        map(json!({ "title": "New", "status": "published" })),
    );
    let saved = log_model_event(
        &except_storage,
        &except_post,
        ActivityEvent::Updated,
        "Post updated",
        full_changes,
        None,
    )
    .await
    .unwrap()
    .unwrap();
    let recorded = saved.attribute_changes.unwrap();
    assert_eq!(recorded.attributes.len(), 1);
    assert!(recorded.attributes.contains_key("title"));

    let only_storage = sqlite_storage().await;
    let only_post = Post {
        id: 1,
        options: LogOptions::default().log_only(["status"]),
    };
    let full_changes = AttributeChanges::new(
        map(json!({ "title": "Old", "status": "draft" })),
        map(json!({ "title": "New", "status": "published" })),
    );
    let saved = log_model_event(
        &only_storage,
        &only_post,
        ActivityEvent::Updated,
        "Post updated",
        full_changes,
        None,
    )
    .await
    .unwrap()
    .unwrap();
    let recorded = saved.attribute_changes.unwrap();
    assert_eq!(recorded.attributes.len(), 1);
    assert!(recorded.attributes.contains_key("status"));
}

#[tokio::test]
async fn empty_logs_are_skipped_by_default() {
    let storage = sqlite_storage().await;
    let post = Post {
        id: 1,
        options: LogOptions::default().log_only_dirty(),
    };
    let unchanged = AttributeChanges::new(
        map(json!({ "title": "Same" })),
        map(json!({ "title": "Same" })),
    );

    let result = log_model_event(
        &storage,
        &post,
        ActivityEvent::Updated,
        "Post updated",
        unchanged,
        None,
    )
    .await
    .unwrap();

    assert!(result.is_none());
    assert_eq!(ActivityQuery::new().count(&storage).await.unwrap(), 0);

    let log_all_disabled = Post {
        id: 1,
        options: LogOptions {
            log_all: false,
            ..LogOptions::default()
        },
    };
    let changes = AttributeChanges::detect(&BTreeMap::new(), &map(json!({ "title": "Hello" })));
    let result = log_model_event(
        &storage,
        &log_all_disabled,
        ActivityEvent::Created,
        "Post created",
        changes,
        None,
    )
    .await
    .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn attribute_changes_keep_only_dirty_values() {
    let old = map(json!({ "title": "Old", "status": "draft", "views": 1 }));
    let new = map(json!({ "title": "New", "status": "draft", "views": 2 }));

    let changes = AttributeChanges::detect(&old, &new);

    assert_eq!(changes.attributes.len(), 2);
    assert_eq!(changes.attributes.get("title"), Some(&json!("New")));
    assert_eq!(changes.attributes.get("views"), Some(&json!(2)));
    assert!(!changes.attributes.contains_key("status"));
    assert_eq!(changes.old.get("title"), Some(&json!("Old")));
    assert_eq!(changes.old.get("views"), Some(&json!(1)));
    assert!(!changes.old.contains_key("status"));
    assert!(changes.new_value("title").is_some());
    assert!(changes.old_value("views").is_some());

    assert_eq!(
        event_detector::detect(&BTreeMap::new(), &new),
        ActivityEvent::Created
    );
}

#[tokio::test]
async fn query_builder_combines_filters() {
    let storage = sqlite_storage().await;
    let base = Utc::now();

    let mut first = record("auth", "first", base);
    first.subject_type = Some("User".to_string());
    first.subject_id = Some("1".to_string());
    first.causer_type = Some("Admin".to_string());
    first.causer_id = Some("9".to_string());
    first.event = Some(ActivityEvent::Created);
    first.batch_uuid = Some("batch-1".to_string());

    let mut second = record("auth", "second", base + Duration::minutes(1));
    second.subject_type = Some("User".to_string());
    second.subject_id = Some("1".to_string());
    second.causer_type = Some("Admin".to_string());
    second.causer_id = Some("9".to_string());
    second.event = Some(ActivityEvent::Custom("login".to_string()));
    second.batch_uuid = Some("batch-1".to_string());

    let mut third = record("posts", "third", base + Duration::minutes(2));
    third.subject_type = Some("Post".to_string());
    third.subject_id = Some("2".to_string());
    third.causer_type = Some("Admin".to_string());
    third.causer_id = Some("9".to_string());
    third.event = Some(ActivityEvent::Updated);

    let mut fourth = record("auth", "fourth", base + Duration::minutes(3));
    fourth.subject_type = Some("User".to_string());
    fourth.subject_id = Some("3".to_string());
    fourth.event = Some(ActivityEvent::Created);

    for item in [&first, &second, &third, &fourth] {
        storage.insert(item).await.unwrap();
    }

    let auth = ActivityQuery::new()
        .in_log("auth")
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(auth.len(), 3);
    assert_eq!(auth[0].id, fourth.id);
    assert_eq!(auth[2].id, first.id);

    let subject_rows = ActivityQuery::new()
        .for_subject(&User { id: 1 })
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(subject_rows.len(), 2);

    let causer_rows = ActivityQuery::new()
        .caused_by(&Admin)
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(causer_rows.len(), 3);

    let created_rows = ActivityQuery::new()
        .for_event(ActivityEvent::Created)
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(created_rows.len(), 2);

    let batch_rows = ActivityQuery::new()
        .in_batch("batch-1")
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(batch_rows.len(), 2);

    let between_rows = ActivityQuery::new()
        .between(base + Duration::seconds(30), base + Duration::seconds(150))
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(between_rows.len(), 2);

    let combined = ActivityQuery::new()
        .in_log("auth")
        .for_subject(&User { id: 1 })
        .for_event(ActivityEvent::Custom("login".to_string()))
        .get(&storage)
        .await
        .unwrap();
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].id, second.id);

    assert_eq!(
        ActivityQuery::new()
            .in_log("auth")
            .count(&storage)
            .await
            .unwrap(),
        3
    );

    let oldest = ActivityQuery::new()
        .oldest()
        .first(&storage)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(oldest.id, first.id);
}

#[tokio::test]
async fn delete_older_than_removes_old_records() {
    let storage = sqlite_storage().await;
    let now = Utc::now();

    storage
        .insert(&record("auth", "old", now - Duration::days(40)))
        .await
        .unwrap();
    storage
        .insert(&record("auth", "recent", now - Duration::days(2)))
        .await
        .unwrap();

    let deleted = delete_older_than(&storage, 30).await.unwrap();

    assert_eq!(deleted, 1);
    let rows = ActivityQuery::new().get(&storage).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].description, "recent");
}

#[derive(Debug, PartialEq)]
struct Signature {
    log_name: String,
    description: String,
    event: Option<String>,
    subject_id: Option<String>,
    causer_id: Option<String>,
    properties: BTreeMap<String, Value>,
    changes: Option<AttributeChanges>,
}

fn signature(activity: &Activity) -> Signature {
    Signature {
        log_name: activity.log_name.clone(),
        description: activity.description.clone(),
        event: activity.event.as_ref().map(|value| value.to_string()),
        subject_id: activity.subject_id.clone(),
        causer_id: activity.causer_id.clone(),
        properties: activity.properties.clone(),
        changes: activity.attribute_changes.clone(),
    }
}

async fn parity_scenario<S: ActivityStorage>(storage: &S) -> Vec<Signature> {
    activity("auth")
        .performed_on(&User { id: 1 })
        .caused_by(&Admin)
        .event(ActivityEvent::Created)
        .with_property("ip", "10.0.0.1")
        .log("first")
        .save(storage)
        .await
        .unwrap();

    activity("auth")
        .performed_on(&User { id: 2 })
        .event(ActivityEvent::Custom("login".to_string()))
        .with_attribute_changes(AttributeChanges::detect(
            &map(json!({ "name": "Old" })),
            &map(json!({ "name": "New" })),
        ))
        .log("second")
        .save(storage)
        .await
        .unwrap();

    let mut rows: Vec<Signature> = ActivityQuery::new()
        .in_log("auth")
        .get(storage)
        .await
        .unwrap()
        .iter()
        .map(signature)
        .collect();
    rows.sort_by(|left, right| left.description.cmp(&right.description));
    rows
}

#[tokio::test]
async fn memory_and_sqlx_storage_agree() {
    let sqlite = sqlite_storage().await;
    let memory = MemoryStorage::new();

    let sqlite_rows = parity_scenario(&sqlite).await;
    let memory_rows = parity_scenario(&memory).await;

    assert_eq!(sqlite_rows, memory_rows);
}
