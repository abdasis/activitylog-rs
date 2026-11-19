//! Event type for an activity record.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Jenis event yang dicatat pada sebuah activity.
///
/// Nilai disimpan sebagai teks pada kolom `event`. Nilai yang tidak dikenal
/// disimpan sebagai [`ActivityEvent::Custom`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActivityEvent {
    /// Record baru dibuat.
    Created,
    /// Record diubah.
    Updated,
    /// Record dihapus.
    Deleted,
    /// Record dipulihkan.
    Restored,
    /// Event bebas, misalnya `login` atau `exported`.
    Custom(String),
}

impl ActivityEvent {
    /// Teks yang disimpan di database dan ditampilkan ke pengguna.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
            Self::Restored => "restored",
            Self::Custom(value) => value,
        }
    }

    /// Membaca event dari teks. Teks tak dikenal menjadi [`ActivityEvent::Custom`].
    pub fn parse(value: &str) -> Self {
        match value {
            "created" => Self::Created,
            "updated" => Self::Updated,
            "deleted" => Self::Deleted,
            "restored" => Self::Restored,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for ActivityEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ActivityEvent {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl From<String> for ActivityEvent {
    fn from(value: String) -> Self {
        Self::parse(&value)
    }
}

impl Serialize for ActivityEvent {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ActivityEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(Self::parse(&value))
    }
}
