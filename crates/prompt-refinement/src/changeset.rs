use crate::error::RefinementError;
use prompt_core::{validate_tag_name, PromptDocument};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// A proposed modification to an existing section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SectionChange {
    pub section_id: Uuid,
    pub refined: String,
    pub reason: String,
}

/// A suggested new section that the model recommends adding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SuggestedSection {
    pub tag: String,
    pub content: String,
    pub reason: String,
}

/// A complete structured refinement response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RefinementChangeset {
    #[serde(default)]
    pub changes: Vec<SectionChange>,
    #[serde(default)]
    pub suggested_sections: Vec<SuggestedSection>,
    #[serde(default)]
    pub open_questions: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub critique: Option<String>,
}

impl RefinementChangeset {
    /// Strictly validates this changeset against a target prompt document.
    pub fn validate_against_document(&self, doc: &PromptDocument) -> Result<(), RefinementError> {
        let mut seen_ids = HashSet::new();

        for change in &self.changes {
            // Check for duplicates
            if !seen_ids.insert(change.section_id) {
                return Err(RefinementError::DuplicateSectionTarget(change.section_id));
            }

            // Check that section exists in document
            let section = doc
                .section(change.section_id)
                .ok_or(RefinementError::UnknownSectionId(change.section_id))?;

            // Check that target section is not locked
            if section.locked {
                return Err(RefinementError::SectionLocked(change.section_id));
            }
        }

        // Validate suggested section tag names
        for suggested in &self.suggested_sections {
            validate_tag_name(&suggested.tag).map_err(|e| {
                RefinementError::InvalidSuggestedTag {
                    tag: suggested.tag.clone(),
                    reason: e.to_string(),
                }
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prompt_core::PromptSection;

    #[test]
    fn test_validate_valid_changeset() {
        let mut doc = PromptDocument::new("Doc", "");
        let sec = PromptSection::new("role", "Initial brief").unwrap();
        let id = doc.add_section(sec).unwrap();

        let changeset = RefinementChangeset {
            changes: vec![SectionChange {
                section_id: id,
                refined: "Refined brief".to_string(),
                reason: "Improves clarity".to_string(),
            }],
            suggested_sections: vec![SuggestedSection {
                tag: "constraints".to_string(),
                content: "No third party dependencies".to_string(),
                reason: "Adds boundary".to_string(),
            }],
            ..Default::default()
        };

        assert!(changeset.validate_against_document(&doc).is_ok());
    }

    #[test]
    fn test_rejects_unknown_id() {
        let doc = PromptDocument::new("Doc", "");
        let changeset = RefinementChangeset {
            changes: vec![SectionChange {
                section_id: Uuid::new_v4(),
                refined: "Text".to_string(),
                reason: "Reason".to_string(),
            }],
            ..Default::default()
        };

        assert!(matches!(
            changeset.validate_against_document(&doc),
            Err(RefinementError::UnknownSectionId(_))
        ));
    }

    #[test]
    fn test_rejects_locked_section() {
        let mut doc = PromptDocument::new("Doc", "");
        let mut sec = PromptSection::new("role", "Initial").unwrap();
        sec.locked = true;
        let id = doc.add_section(sec).unwrap();

        let changeset = RefinementChangeset {
            changes: vec![SectionChange {
                section_id: id,
                refined: "Text".to_string(),
                reason: "Reason".to_string(),
            }],
            ..Default::default()
        };

        assert!(matches!(
            changeset.validate_against_document(&doc),
            Err(RefinementError::SectionLocked(sec_id)) if sec_id == id
        ));
    }

    #[test]
    fn test_rejects_duplicate_targets() {
        let mut doc = PromptDocument::new("Doc", "");
        let sec = PromptSection::new("role", "Initial").unwrap();
        let id = doc.add_section(sec).unwrap();

        let changeset = RefinementChangeset {
            changes: vec![
                SectionChange {
                    section_id: id,
                    refined: "First".to_string(),
                    reason: "Reason 1".to_string(),
                },
                SectionChange {
                    section_id: id,
                    refined: "Second".to_string(),
                    reason: "Reason 2".to_string(),
                },
            ],
            ..Default::default()
        };

        assert!(matches!(
            changeset.validate_against_document(&doc),
            Err(RefinementError::DuplicateSectionTarget(sec_id)) if sec_id == id
        ));
    }
}
