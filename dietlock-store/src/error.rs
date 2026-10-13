use std::fmt;

use dietlock_core::ValidationError;

/// Database access error.
#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    /// Schema newer than this version of the application.
    UnsupportedSchemaVersion(i32),
    /// Row read from the database that fails type validation.
    InvalidRow(ValidationError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "SQLite error: {e}"),
            Self::UnsupportedSchemaVersion(v) => write!(f, "unknown schema version: {v}"),
            Self::InvalidRow(e) => write!(f, "invalid row in database: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

/// Error returned when saving a weigh-in or an intake.
#[derive(Debug)]
pub enum SaveError<T> {
    /// The date is already filled; holds the existing value.
    AlreadyFilled(T),
    Store(StoreError),
}

impl<T: fmt::Debug> fmt::Display for SaveError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyFilled(old) => write!(f, "already filled: {old:?}"),
            Self::Store(e) => e.fmt(f),
        }
    }
}

impl<T: fmt::Debug> std::error::Error for SaveError<T> {}

impl<T> From<StoreError> for SaveError<T> {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

impl<T> From<rusqlite::Error> for SaveError<T> {
    fn from(e: rusqlite::Error) -> Self {
        Self::Store(e.into())
    }
}
