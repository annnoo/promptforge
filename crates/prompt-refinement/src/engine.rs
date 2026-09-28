use crate::changeset::RefinementChangeset;
use crate::error::RefinementError;
use crate::prompts::RefinementMode;
use crate::provider::RefinementProvider;
use prompt_core::PromptDocument;

/// Refinement engine coordinating model execution and validation.
#[derive(Debug, Default)]
pub struct RefinementEngine;

impl RefinementEngine {
    pub fn new() -> Self {
        Self
    }

    /// Executes refinement using the given provider and mode, ensuring the returned changeset is valid.
    pub async fn execute(
        &self,
        provider: &dyn RefinementProvider,
        doc: &PromptDocument,
        mode: RefinementMode,
    ) -> Result<RefinementChangeset, RefinementError> {
        let changeset = provider.refine(doc, mode).await?;
        changeset.validate_against_document(doc)?;
        Ok(changeset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;
    use prompt_core::PromptSection;

    #[tokio::test]
    async fn test_engine_with_mock() {
        let mut doc = PromptDocument::new("Doc", "");
        let sec = PromptSection::new("role", "Architect").unwrap();
        let id = doc.add_section(sec).unwrap();

        let engine = RefinementEngine::new();
        let provider = MockProvider::new();

        let res = engine
            .execute(&provider, &doc, RefinementMode::Conservative)
            .await
            .unwrap();

        assert_eq!(res.changes.len(), 1);
        assert_eq!(res.changes[0].section_id, id);
    }
}
