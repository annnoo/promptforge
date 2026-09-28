use prompt_core::CoreError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RefinementError {
    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Invalid changeset payload: {0}")]
    InvalidPayload(String),

    #[error("Unknown section ID '{0}' targeted in refinement")]
    UnknownSectionId(Uuid),

    #[error("Section '{0}' targeted in refinement is locked")]
    SectionLocked(Uuid),

    #[error("Duplicate changes targeting section '{0}'")]
    DuplicateSectionTarget(Uuid),

    #[error("Invalid suggested section tag '{tag}': {reason}")]
    InvalidSuggestedTag { tag: String, reason: String },

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Missing API key environment variable: '{0}'")]
    MissingApiKey(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
