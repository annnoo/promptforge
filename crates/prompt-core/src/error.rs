use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    #[error("Invalid XML tag name '{tag}': {reason}")]
    InvalidTagName { tag: String, reason: String },

    #[error("Section not found: {0}")]
    SectionNotFound(Uuid),

    #[error("Section '{0}' is locked and cannot be modified")]
    SectionLocked(Uuid),

    #[error("Duplicate section ID found: {0}")]
    DuplicateSectionId(Uuid),

    #[error("Index out of bounds: index {index}, len {len}")]
    IndexOutOfBounds { index: usize, len: usize },

    #[error("XML serialization error: {0}")]
    XmlSerializationError(String),

    #[error("XML parsing error: {0}")]
    XmlParsingError(String),

    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },

    #[error("Validation error: {0}")]
    Validation(String),
}
