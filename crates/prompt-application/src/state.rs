use prompt_core::PromptDocument;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// A single proposed text modification for an existing section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedSectionChange {
    pub section_id: Uuid,
    pub proposed_text: String,
    pub reason: String,
}

/// A suggested new section that the model recommends adding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedSection {
    pub tag: String,
    pub content: String,
    pub reason: String,
}

/// A pending set of refinements awaiting user review and accept/reject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PendingRefinements {
    pub changes: Vec<ProposedSectionChange>,
    pub suggested_sections: Vec<SuggestedSection>,
    pub open_questions: Vec<String>,
    pub warnings: Vec<String>,
    pub critique: Option<String>,
}

impl PendingRefinements {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
            && self.suggested_sections.is_empty()
            && self.open_questions.is_empty()
            && self.warnings.is_empty()
            && self.critique.is_none()
    }

    pub fn change_for_section(&self, id: Uuid) -> Option<&ProposedSectionChange> {
        self.changes.iter().find(|c| c.section_id == id)
    }

    pub fn remove_change(&mut self, id: Uuid) -> Option<ProposedSectionChange> {
        if let Some(pos) = self.changes.iter().position(|c| c.section_id == id) {
            Some(self.changes.remove(pos))
        } else {
            None
        }
    }
}

/// Core domain & application state held by the application service.
#[derive(Debug, Clone, Default)]
pub struct ApplicationState {
    /// Active prompt document.
    pub document: PromptDocument,
    /// Whether there are unsaved changes since last save/load.
    pub dirty: bool,
    /// Currently loaded or saved file path, if any.
    pub current_file_path: Option<PathBuf>,
    /// Pending refinements awaiting user acceptance.
    pub pending_refinements: Option<PendingRefinements>,
}

impl ApplicationState {
    pub fn new(doc: PromptDocument) -> Self {
        Self {
            document: doc,
            dirty: false,
            current_file_path: None,
            pending_refinements: None,
        }
    }
}
