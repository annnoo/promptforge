use prompt_core::CoreError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse JSON file '{path}': {source}")]
    JsonDeserialization {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to serialize document to JSON: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("Schema version {found} is newer than supported version {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },

    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Invalid document structure in '{path}': {reason}")]
    InvalidDocument { path: PathBuf, reason: String },

    #[error("Configuration error: {0}")]
    Config(String),
}
