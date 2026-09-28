use crate::error::CoreError;
use crate::validation::validate_tag_name;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stage of prompt rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RenderStage {
    #[default]
    Draft,
    Final,
}

/// A structured section in a PromptDocument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSection {
    /// Unique immutable identifier for this section.
    pub id: Uuid,
    /// XML tag name for this section (e.g., "role", "task", "constraints").
    pub tag: String,
    /// User's original notes or initial prompt instructions.
    pub brief: String,
    /// Refined version accepted from an LLM or manual refinement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refined: Option<String>,
    /// Whether the section is locked against automated or accidental modification.
    #[serde(default)]
    pub locked: bool,
    /// Whether the section is enabled in generated output.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl PromptSection {
    /// Creates a new prompt section with a validated XML tag and auto-generated UUID.
    pub fn new(tag: impl Into<String>, brief: impl Into<String>) -> Result<Self, CoreError> {
        let tag = tag.into();
        validate_tag_name(&tag)?;
        Ok(Self {
            id: Uuid::new_v4(),
            tag,
            brief: brief.into(),
            refined: None,
            locked: false,
            enabled: true,
        })
    }

    /// Creates a new prompt section with a specific UUID and validated XML tag.
    pub fn with_id(
        id: Uuid,
        tag: impl Into<String>,
        brief: impl Into<String>,
    ) -> Result<Self, CoreError> {
        let tag = tag.into();
        validate_tag_name(&tag)?;
        Ok(Self {
            id,
            tag,
            brief: brief.into(),
            refined: None,
            locked: false,
            enabled: true,
        })
    }

    /// Returns the content to be rendered for a given stage.
    /// In Draft stage, always returns `brief`.
    /// In Final stage, returns `refined` if present, otherwise returns `brief`.
    pub fn effective_content(&self, stage: RenderStage) -> &str {
        match stage {
            RenderStage::Draft => &self.brief,
            RenderStage::Final => self.refined.as_deref().unwrap_or(&self.brief),
        }
    }

    /// Returns true if this section has an accepted refined version.
    pub fn has_refinement(&self) -> bool {
        self.refined.is_some()
    }

    /// Renames the XML tag after validation.
    pub fn rename_tag(&mut self, new_tag: impl Into<String>) -> Result<(), CoreError> {
        let new_tag = new_tag.into();
        validate_tag_name(&new_tag)?;
        self.tag = new_tag;
        Ok(())
    }

    /// Clones the section with a new unique UUID.
    pub fn duplicate(&self) -> Self {
        Self {
            id: Uuid::new_v4(),
            tag: self.tag.clone(),
            brief: self.brief.clone(),
            refined: self.refined.clone(),
            locked: false,
            enabled: self.enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_section() {
        let sec = PromptSection::new("role", "Senior backend engineer").unwrap();
        assert_eq!(sec.tag, "role");
        assert_eq!(sec.brief, "Senior backend engineer");
        assert_eq!(sec.refined, None);
        assert!(!sec.locked);
        assert!(sec.enabled);
    }

    #[test]
    fn test_effective_content() {
        let mut sec = PromptSection::new("task", "Draft task").unwrap();
        assert_eq!(sec.effective_content(RenderStage::Draft), "Draft task");
        assert_eq!(sec.effective_content(RenderStage::Final), "Draft task");

        sec.refined = Some("Polished task".to_string());
        assert_eq!(sec.effective_content(RenderStage::Draft), "Draft task");
        assert_eq!(sec.effective_content(RenderStage::Final), "Polished task");
    }

    #[test]
    fn test_duplicate_generates_new_id() {
        let sec1 = PromptSection::new("context", "Monolith repo").unwrap();
        let sec2 = sec1.duplicate();
        assert_ne!(sec1.id, sec2.id);
        assert_eq!(sec1.tag, sec2.tag);
        assert_eq!(sec1.brief, sec2.brief);
    }
}
