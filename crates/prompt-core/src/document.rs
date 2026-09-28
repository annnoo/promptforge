use crate::error::CoreError;
use crate::section::PromptSection;
use crate::validation::{validate_tag_name, validate_unique_section_ids};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// A PromptForge prompt document containing metadata and an ordered list of structured sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptDocument {
    /// Schema version for migrations and persistence compatibility.
    pub schema_version: u32,
    /// Unique document ID.
    pub id: Uuid,
    /// Title of the prompt document.
    pub title: String,
    /// Description or purpose of this prompt.
    pub description: String,
    /// Ordered list of prompt sections.
    pub sections: Vec<PromptSection>,
}

impl Default for PromptDocument {
    fn default() -> Self {
        Self::new("Untitled Prompt", "")
    }
}

impl PromptDocument {
    /// Creates a new empty document with the default schema version.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            sections: Vec::new(),
        }
    }

    /// Validates the document invariants:
    /// - All section tags are valid XML elements
    /// - All section IDs are unique
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_unique_section_ids(self.sections.iter().map(|s| &s.id))?;
        for section in &self.sections {
            validate_tag_name(&section.tag)?;
        }
        Ok(())
    }

    /// Adds a section to the end of the document.
    pub fn add_section(&mut self, section: PromptSection) -> Result<Uuid, CoreError> {
        validate_tag_name(&section.tag)?;
        if self.sections.iter().any(|s| s.id == section.id) {
            return Err(CoreError::DuplicateSectionId(section.id));
        }
        let id = section.id;
        self.sections.push(section);
        Ok(id)
    }

    /// Inserts a section at the specified index.
    pub fn insert_section(&mut self, index: usize, section: PromptSection) -> Result<Uuid, CoreError> {
        validate_tag_name(&section.tag)?;
        if index > self.sections.len() {
            return Err(CoreError::IndexOutOfBounds {
                index,
                len: self.sections.len(),
            });
        }
        if self.sections.iter().any(|s| s.id == section.id) {
            return Err(CoreError::DuplicateSectionId(section.id));
        }
        let id = section.id;
        self.sections.insert(index, section);
        Ok(id)
    }

    /// Removes a section by ID, returning it if found.
    pub fn remove_section(&mut self, id: Uuid) -> Option<PromptSection> {
        let index = self.section_index(id)?;
        Some(self.sections.remove(index))
    }

    /// Duplicates a section and inserts the duplicate immediately after it.
    pub fn duplicate_section(&mut self, id: Uuid) -> Option<PromptSection> {
        let index = self.section_index(id)?;
        let duplicate = self.sections[index].duplicate();
        let dup_clone = duplicate.clone();
        self.sections.insert(index + 1, duplicate);
        Some(dup_clone)
    }

    /// Moves a section from index `from` to index `to`.
    pub fn move_section(&mut self, from: usize, to: usize) -> Result<(), CoreError> {
        let len = self.sections.len();
        if from >= len {
            return Err(CoreError::IndexOutOfBounds { index: from, len });
        }
        if to >= len {
            return Err(CoreError::IndexOutOfBounds { index: to, len });
        }
        if from == to {
            return Ok(());
        }
        let item = self.sections.remove(from);
        self.sections.insert(to, item);
        Ok(())
    }

    /// Moves a section with ID `id` to the new index `to`.
    pub fn move_section_by_id(&mut self, id: Uuid, to: usize) -> Result<(), CoreError> {
        let from = self.section_index(id).ok_or(CoreError::SectionNotFound(id))?;
        self.move_section(from, to)
    }

    /// Moves a section one position up.
    pub fn reorder_up(&mut self, id: Uuid) -> bool {
        if let Some(idx) = self.section_index(id) {
            if idx > 0 {
                self.sections.swap(idx, idx - 1);
                return true;
            }
        }
        false
    }

    /// Moves a section one position down.
    pub fn reorder_down(&mut self, id: Uuid) -> bool {
        if let Some(idx) = self.section_index(id) {
            if idx + 1 < self.sections.len() {
                self.sections.swap(idx, idx + 1);
                return true;
            }
        }
        false
    }

    /// Finds a section by its ID.
    pub fn section(&self, id: Uuid) -> Option<&PromptSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Finds a mutable reference to a section by its ID.
    pub fn section_mut(&mut self, id: Uuid) -> Option<&mut PromptSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    /// Returns the index of a section by its ID.
    pub fn section_index(&self, id: Uuid) -> Option<usize> {
        self.sections.iter().position(|s| s.id == id)
    }

    /// Returns all enabled sections.
    pub fn enabled_sections(&self) -> impl Iterator<Item = &PromptSection> {
        self.sections.iter().filter(|s| s.enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_add_and_remove() {
        let mut doc = PromptDocument::new("Test Doc", "A test document");
        let s1 = PromptSection::new("role", "Developer").unwrap();
        let s2 = PromptSection::new("task", "Code").unwrap();

        let id1 = doc.add_section(s1).unwrap();
        let id2 = doc.add_section(s2).unwrap();

        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.section(id1).unwrap().tag, "role");

        let removed = doc.remove_section(id1).unwrap();
        assert_eq!(removed.tag, "role");
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].id, id2);
    }

    #[test]
    fn test_reorder_sections() {
        let mut doc = PromptDocument::new("Reorder Test", "");
        let s1 = PromptSection::new("first", "1").unwrap();
        let s2 = PromptSection::new("second", "2").unwrap();
        let s3 = PromptSection::new("third", "3").unwrap();

        let id1 = doc.add_section(s1).unwrap();
        let id2 = doc.add_section(s2).unwrap();
        let id3 = doc.add_section(s3).unwrap();

        doc.reorder_down(id1);
        assert_eq!(doc.sections[0].id, id2);
        assert_eq!(doc.sections[1].id, id1);
        assert_eq!(doc.sections[2].id, id3);

        doc.move_section_by_id(id3, 0).unwrap();
        assert_eq!(doc.sections[0].id, id3);
        assert_eq!(doc.sections[1].id, id2);
        assert_eq!(doc.sections[2].id, id1);
    }

    #[test]
    fn test_duplicate_section() {
        let mut doc = PromptDocument::new("Dup Test", "");
        let s1 = PromptSection::new("role", "Developer").unwrap();
        let id1 = doc.add_section(s1).unwrap();

        let dup = doc.duplicate_section(id1).unwrap();
        assert_eq!(doc.sections.len(), 2);
        assert_ne!(dup.id, id1);
        assert_eq!(dup.tag, "role");
        assert_eq!(doc.sections[1].id, dup.id);
    }
}
