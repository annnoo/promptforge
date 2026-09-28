use prompt_core::CoreError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Cannot undo: no previous history")]
    CannotUndo,

    #[error("Cannot redo: already at newest state")]
    CannotRedo,

    #[error("Section not found: {0}")]
    SectionNotFound(Uuid),

    #[error("Cannot modify locked section: {0}")]
    SectionLocked(Uuid),

    #[error("No pending refinement found for section: {0}")]
    NoPendingRefinement(Uuid),

    #[error("Application error: {0}")]
    General(String),
}
