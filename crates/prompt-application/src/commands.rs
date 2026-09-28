use uuid::Uuid;

/// Commands that modify domain and application state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Add a new section with a given XML tag and initial brief.
    AddSection { tag: String, brief: String },
    /// Add a section from a preset tag name.
    AddPreset { preset_tag: String },
    /// Remove an existing section by ID.
    RemoveSection { id: Uuid },
    /// Duplicate an existing section by ID.
    DuplicateSection { id: Uuid },
    /// Move a section to a specific index position.
    MoveSection { id: Uuid, to: usize },
    /// Move a section one slot up.
    MoveUp { id: Uuid },
    /// Move a section one slot down.
    MoveDown { id: Uuid },
    /// Rename a section's XML tag.
    RenameSection { id: Uuid, tag: String },
    /// Update the original brief text of a section.
    UpdateBrief { id: Uuid, text: String },
    /// Update document title.
    UpdateTitle { title: String },
    /// Update document description.
    UpdateDescription { description: String },
    /// Toggle the locked state of a section.
    ToggleLock { id: Uuid },
    /// Toggle the enabled state of a section.
    ToggleEnabled { id: Uuid },
    /// Directly set or clear the refined text for a section.
    SetRefined { id: Uuid, refined: Option<String> },
    /// Clear the accepted refinement for a section.
    ClearRefinement { id: Uuid },
    /// Apply a proposed refinement for a specific section.
    ApplyRefinement { id: Uuid },
    /// Reject a proposed refinement for a specific section.
    RejectRefinement { id: Uuid },
    /// Apply all pending proposed refinements.
    ApplyAllRefinements,
    /// Reject all pending proposed refinements.
    RejectAllRefinements,
    /// Undo the last discrete operation.
    Undo,
    /// Redo the last undone operation.
    Redo,
}
