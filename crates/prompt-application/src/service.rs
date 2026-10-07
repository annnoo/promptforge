use crate::commands::Command;
use crate::error::ApplicationError;
use crate::history::DocumentHistory;
use crate::state::{ApplicationState, PendingRefinements};
use prompt_core::{template::SectionPreset, PromptDocument, PromptSection, RenderOptions};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Application service mediating all domain mutations, undo/redo, and refinement review.
#[derive(Debug, Clone)]
pub struct ApplicationService {
    state: ApplicationState,
    history: DocumentHistory,
}

impl Default for ApplicationService {
    fn default() -> Self {
        Self::new(PromptDocument::default())
    }
}

impl ApplicationService {
    /// Creates a new service with the given document.
    pub fn new(document: PromptDocument) -> Self {
        Self {
            state: ApplicationState::new(document),
            history: DocumentHistory::default(),
        }
    }

    /// Access the current application state.
    pub fn state(&self) -> &ApplicationState {
        &self.state
    }

    /// Access mutable application state.
    pub fn state_mut(&mut self) -> &mut ApplicationState {
        &mut self.state
    }

    /// Access the active document.
    pub fn document(&self) -> &PromptDocument {
        &self.state.document
    }

    /// Returns whether there are unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.state.dirty
    }

    /// Returns the currently active file path.
    pub fn current_file_path(&self) -> Option<&Path> {
        self.state.current_file_path.as_deref()
    }

    /// Returns whether undo is available.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Returns whether redo is available.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Performs undo if available, returning true if successful.
    pub fn undo(&mut self) -> bool {
        self.execute(Command::Undo).is_ok()
    }

    /// Performs redo if available, returning true if successful.
    pub fn redo(&mut self) -> bool {
        self.execute(Command::Redo).is_ok()
    }

    /// Marks the active document as saved to the given path.
    pub fn mark_saved(&mut self, path: PathBuf) {
        self.state.current_file_path = Some(path);
        self.state.dirty = false;
    }

    /// Sets or replaces the active document (e.g. on open or new), resetting history.
    pub fn load_document(&mut self, document: PromptDocument, path: Option<PathBuf>) {
        self.state.document = document;
        self.state.current_file_path = path;
        self.state.dirty = false;
        self.state.pending_refinements = None;
        self.history.clear();
    }

    /// Sets pending refinements for user review.
    pub fn set_pending_refinements(&mut self, pending: PendingRefinements) {
        self.state.pending_refinements = Some(pending);
    }

    /// Clears any pending refinements.
    pub fn clear_pending_refinements(&mut self) {
        self.state.pending_refinements = None;
    }

    /// Pushes the current document state into history as a checkpoint.
    pub fn commit_checkpoint(&mut self) {
        self.history.push_snapshot(self.state.document.clone());
    }

    /// Live updates a section's brief without immediately creating a history snapshot.
    /// This prevents every individual keystroke from bloating the undo stack.
    pub fn update_brief_live(&mut self, id: Uuid, text: String) -> Result<(), ApplicationError> {
        let section = self
            .state
            .document
            .section_mut(id)
            .ok_or(ApplicationError::SectionNotFound(id))?;

        if section.locked {
            return Err(ApplicationError::SectionLocked(id));
        }

        if section.brief != text {
            section.brief = text;
            self.state.dirty = true;
        }
        Ok(())
    }

    /// Appends multiple sections to the active document with a history checkpoint.
    pub fn append_sections(&mut self, sections: Vec<PromptSection>) -> Result<(), ApplicationError> {
        if sections.is_empty() {
            return Ok(());
        }
        self.commit_checkpoint();
        for section in sections {
            self.state.document.add_section(section)?;
        }
        self.state.dirty = true;
        Ok(())
    }

    /// Executes a command against the application state.
    pub fn execute(&mut self, command: Command) -> Result<(), ApplicationError> {
        match command {
            Command::AddSection { tag, brief } => {
                let section = PromptSection::new(tag, brief)?;
                self.commit_checkpoint();
                self.state.document.add_section(section)?;
                self.state.dirty = true;
            }
            Command::AddPreset { preset_tag } => {
                let preset = SectionPreset::find_by_tag(&preset_tag).ok_or_else(|| {
                    ApplicationError::General(format!("Unknown section preset '{preset_tag}'"))
                })?;
                let section = preset.to_section();
                self.commit_checkpoint();
                self.state.document.add_section(section)?;
                self.state.dirty = true;
            }
            Command::RemoveSection { id } => {
                let section = self
                    .state
                    .document
                    .section(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }
                self.commit_checkpoint();
                self.state.document.remove_section(id);
                if let Some(ref mut pending) = self.state.pending_refinements {
                    pending.remove_change(id);
                }
                self.state.dirty = true;
            }
            Command::DuplicateSection { id } => {
                self.commit_checkpoint();
                self.state
                    .document
                    .duplicate_section(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                self.state.dirty = true;
            }
            Command::MoveSection { id, to } => {
                self.commit_checkpoint();
                self.state.document.move_section_by_id(id, to)?;
                self.state.dirty = true;
            }
            Command::MoveUp { id } => {
                self.commit_checkpoint();
                if self.state.document.reorder_up(id) {
                    self.state.dirty = true;
                }
            }
            Command::MoveDown { id } => {
                self.commit_checkpoint();
                if self.state.document.reorder_down(id) {
                    self.state.dirty = true;
                }
            }
            Command::RenameSection { id, tag } => {
                let section = self
                    .state
                    .document
                    .section_mut(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }
                self.commit_checkpoint();
                // re-borrow mutably after checkpoint
                let section = self.state.document.section_mut(id).unwrap();
                section.rename_tag(tag)?;
                self.state.dirty = true;
            }
            Command::UpdateBrief { id, text } => {
                let section = self
                    .state
                    .document
                    .section_mut(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }
                if section.brief != text {
                    self.commit_checkpoint();
                    let section = self.state.document.section_mut(id).unwrap();
                    section.brief = text;
                    self.state.dirty = true;
                }
            }
            Command::UpdateTags { id, tags } => {
                let section = self
                    .state
                    .document
                    .section_mut(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }
                if section.tags != tags {
                    self.commit_checkpoint();
                    let section = self.state.document.section_mut(id).unwrap();
                    section.tags = tags;
                    self.state.dirty = true;
                }
            }
            Command::UpdateTitle { title } => {
                if self.state.document.title != title {
                    self.commit_checkpoint();
                    self.state.document.title = title;
                    self.state.dirty = true;
                }
            }
            Command::UpdateDescription { description } => {
                if self.state.document.description != description {
                    self.commit_checkpoint();
                    self.state.document.description = description;
                    self.state.dirty = true;
                }
            }
            Command::ToggleLock { id } => {
                if !self.state.document.sections.iter().any(|s| s.id == id) {
                    return Err(ApplicationError::SectionNotFound(id));
                }
                self.commit_checkpoint();
                let section = self.state.document.section_mut(id).unwrap();
                section.locked = !section.locked;
                self.state.dirty = true;
            }
            Command::ToggleEnabled { id } => {
                if !self.state.document.sections.iter().any(|s| s.id == id) {
                    return Err(ApplicationError::SectionNotFound(id));
                }
                self.commit_checkpoint();
                let section = self.state.document.section_mut(id).unwrap();
                section.enabled = !section.enabled;
                self.state.dirty = true;
            }
            Command::SetRefined { id, refined } => {
                let section = self
                    .state
                    .document
                    .section_mut(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }
                self.commit_checkpoint();
                let section = self.state.document.section_mut(id).unwrap();
                section.refined = refined;
                self.state.dirty = true;
            }
            Command::ClearRefinement { id } => {
                let section = self
                    .state
                    .document
                    .section_mut(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }
                self.commit_checkpoint();
                let section = self.state.document.section_mut(id).unwrap();
                section.refined = None;
                self.state.dirty = true;
            }
            Command::ApplyRefinement { id } => {
                let section = self
                    .state
                    .document
                    .section_mut(id)
                    .ok_or(ApplicationError::SectionNotFound(id))?;
                if section.locked {
                    return Err(ApplicationError::SectionLocked(id));
                }

                let pending = self
                    .state
                    .pending_refinements
                    .as_mut()
                    .ok_or(ApplicationError::NoPendingRefinement(id))?;

                let change = pending
                    .remove_change(id)
                    .ok_or(ApplicationError::NoPendingRefinement(id))?;

                self.commit_checkpoint();
                let section = self.state.document.section_mut(id).unwrap();
                section.refined = Some(change.proposed_text);
                self.state.dirty = true;
            }
            Command::RejectRefinement { id } => {
                if let Some(ref mut pending) = self.state.pending_refinements {
                    pending.remove_change(id);
                }
            }
            Command::ApplyAllRefinements => {
                if let Some(mut pending) = self.state.pending_refinements.take() {
                    let has_applicable = pending.changes.iter().any(|c| {
                        self.state
                            .document
                            .section(c.section_id)
                            .is_some_and(|s| !s.locked)
                    });

                    if has_applicable {
                        self.commit_checkpoint();
                        for change in pending.changes.drain(..) {
                            if let Some(sec) = self.state.document.section_mut(change.section_id) {
                                if !sec.locked {
                                    sec.refined = Some(change.proposed_text);
                                }
                            }
                        }
                        self.state.dirty = true;
                    }

                    if !pending.is_empty() {
                        self.state.pending_refinements = Some(pending);
                    }
                }
            }
            Command::RejectAllRefinements => {
                if let Some(ref mut pending) = self.state.pending_refinements {
                    pending.changes.clear();
                }
            }
            Command::Undo => {
                let current = self.state.document.clone();
                let restored = self.history.undo(current).ok_or(ApplicationError::CannotUndo)?;
                self.state.document = restored;
                self.state.dirty = true;
            }
            Command::Redo => {
                let current = self.state.document.clone();
                let restored = self.history.redo(current).ok_or(ApplicationError::CannotRedo)?;
                self.state.document = restored;
                self.state.dirty = true;
            }
        }
        Ok(())
    }

    /// Accepts a suggested section proposed by the refinement model, adding it to the document.
    pub fn accept_suggested_section(&mut self, index: usize) -> Result<Uuid, ApplicationError> {
        let pending = self
            .state
            .pending_refinements
            .as_mut()
            .ok_or_else(|| ApplicationError::General("No pending refinements".to_string()))?;

        if index >= pending.suggested_sections.len() {
            return Err(ApplicationError::General(format!(
                "Invalid suggested section index {index}"
            )));
        }

        let suggestion = pending.suggested_sections.remove(index);
        let section = PromptSection::new(suggestion.tag, suggestion.content)?;
        self.commit_checkpoint();
        let id = self.state.document.add_section(section)?;
        self.state.dirty = true;
        Ok(id)
    }

    /// Dismisses a suggested section without adding it to the document.
    pub fn dismiss_suggested_section(&mut self, index: usize) -> Result<(), ApplicationError> {
        let pending = self
            .state
            .pending_refinements
            .as_mut()
            .ok_or_else(|| ApplicationError::General("No pending refinements".to_string()))?;

        if index >= pending.suggested_sections.len() {
            return Err(ApplicationError::General(format!(
                "Invalid suggested section index {index}"
            )));
        }

        pending.suggested_sections.remove(index);
        Ok(())
    }

    /// Renders the document XML according to stage and identity options.
    pub fn render_xml(&self, options: RenderOptions) -> Result<String, ApplicationError> {
        prompt_core::render_xml(&self.state.document, options).map_err(ApplicationError::Core)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProposedSectionChange;

    #[test]
    fn test_execute_add_and_undo() {
        let mut service = ApplicationService::default();
        service
            .execute(Command::AddSection {
                tag: "role".to_string(),
                brief: "Engineer".to_string(),
            })
            .unwrap();

        assert_eq!(service.document().sections.len(), 1);
        assert!(service.is_dirty());
        assert!(service.can_undo());

        service.execute(Command::Undo).unwrap();
        assert_eq!(service.document().sections.len(), 0);
        assert!(service.can_redo());

        service.execute(Command::Redo).unwrap();
        assert_eq!(service.document().sections.len(), 1);
    }

    #[test]
    fn test_locked_section_protection() {
        let mut service = ApplicationService::default();
        service
            .execute(Command::AddSection {
                tag: "role".to_string(),
                brief: "Engineer".to_string(),
            })
            .unwrap();

        let id = service.document().sections[0].id;
        service.execute(Command::ToggleLock { id }).unwrap();
        assert!(service.document().section(id).unwrap().locked);

        // Attempting to update brief of locked section fails
        let res = service.execute(Command::UpdateBrief {
            id,
            text: "Changed".to_string(),
        });
        assert!(res.is_err());

        // Attempting to live-update brief of locked section fails
        let res = service.update_brief_live(id, "Changed live".to_string());
        assert!(res.is_err());
    }

    #[test]
    fn test_apply_and_reject_refinement() {
        let mut service = ApplicationService::default();
        service
            .execute(Command::AddSection {
                tag: "task".to_string(),
                brief: "Initial task".to_string(),
            })
            .unwrap();

        let id = service.document().sections[0].id;

        service.set_pending_refinements(PendingRefinements {
            changes: vec![ProposedSectionChange {
                section_id: id,
                proposed_text: "Refined polished task".to_string(),
                reason: "Adds clarity".to_string(),
            }],
            ..Default::default()
        });

        // Apply refinement
        service.execute(Command::ApplyRefinement { id }).unwrap();
        let sec = service.document().section(id).unwrap();
        assert_eq!(sec.refined.as_deref(), Some("Refined polished task"));
        // Original brief is untouched
        assert_eq!(sec.brief, "Initial task");
        // Pending changes should now be empty
        assert!(service.state().pending_refinements.as_ref().unwrap().changes.is_empty());
    }

    #[test]
    fn test_update_tags_command() {
        let mut service = ApplicationService::default();
        service
            .execute(Command::AddSection {
                tag: "role".to_string(),
                brief: "Engineer".to_string(),
            })
            .unwrap();
        let id = service.document().sections[0].id;

        service
            .execute(Command::UpdateTags {
                id,
                tags: vec!["critical".to_string(), "backend".to_string()],
            })
            .unwrap();

        assert_eq!(
            service.document().section(id).unwrap().tags,
            vec!["critical".to_string(), "backend".to_string()]
        );

        // Test undo
        service.execute(Command::Undo).unwrap();
        assert!(service.document().section(id).unwrap().tags.is_empty());

        // Test redo
        service.execute(Command::Redo).unwrap();
        assert_eq!(
            service.document().section(id).unwrap().tags,
            vec!["critical".to_string(), "backend".to_string()]
        );
    }

    #[test]
    fn test_append_sections() {
        let mut service = ApplicationService::default();
        let s1 = PromptSection::new("role", "Architect").unwrap();
        let s2 = PromptSection::new("task", "Build Engine").unwrap();
        service.append_sections(vec![s1, s2]).unwrap();

        assert_eq!(service.document().sections.len(), 2);
        assert_eq!(service.document().sections[0].tag, "role");
        assert_eq!(service.document().sections[1].tag, "task");
        assert!(service.state().dirty);

        // Can undo the whole append
        service.undo();
        assert_eq!(service.document().sections.len(), 0);
    }
}

