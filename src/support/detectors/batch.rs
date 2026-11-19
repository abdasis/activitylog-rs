//! Batch UUID untuk mengelompokkan activity dalam satu request.

/// Batch yang membawa satu UUID dan bisa dipakai berkali-kali.
#[derive(Debug, Clone)]
pub struct Batch {
    uuid: String,
}

impl Batch {
    /// Membuat batch baru dengan UUID v4.
    pub fn new() -> Self {
        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// UUID batch.
    pub fn uuid(&self) -> &str {
        &self.uuid
    }
}

impl Default for Batch {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Batch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.uuid)
    }
}
