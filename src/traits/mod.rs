//! Trait untuk pencatatan otomatis pada model.

pub mod causes_activity;
pub mod logs_activity;

pub use causes_activity::CausesActivity;
pub use logs_activity::{log_model_event, LogsActivity};
