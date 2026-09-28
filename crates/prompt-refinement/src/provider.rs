use crate::changeset::{RefinementChangeset, SectionChange};
use crate::error::RefinementError;
use crate::prompts::RefinementMode;
use prompt_core::PromptDocument;
use std::future::Future;
use std::pin::Pin;

/// Trait implemented by LLM refinement backends.
pub trait RefinementProvider: Send + Sync {
    /// Refines a document asynchronously.
    fn refine<'a>(
        &'a self,
        doc: &'a PromptDocument,
        mode: RefinementMode,
    ) -> Pin<Box<dyn Future<Output = Result<RefinementChangeset, RefinementError>> + Send + 'a>>;
}

/// A mock provider for deterministic automated testing.
#[derive(Debug, Clone, Default)]
pub struct MockProvider {
    pub custom_changeset: Option<RefinementChangeset>,
    pub should_fail: bool,
}

impl MockProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_changeset(changeset: RefinementChangeset) -> Self {
        Self {
            custom_changeset: Some(changeset),
            should_fail: false,
        }
    }

    pub fn with_failure() -> Self {
        Self {
            custom_changeset: None,
            should_fail: true,
        }
    }
}

impl RefinementProvider for MockProvider {
    fn refine<'a>(
        &'a self,
        doc: &'a PromptDocument,
        mode: RefinementMode,
    ) -> Pin<Box<dyn Future<Output = Result<RefinementChangeset, RefinementError>> + Send + 'a>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(RefinementError::ProviderError("Mock error".into()));
            }

            if let Some(ref cs) = self.custom_changeset {
                cs.validate_against_document(doc)?;
                return Ok(cs.clone());
            }

            // Generate deterministic mock refinement for all unlocked enabled sections
            let mut changes = Vec::new();
            for sec in doc.enabled_sections() {
                if !sec.locked {
                    changes.push(SectionChange {
                        section_id: sec.id,
                        refined: format!("[{}] {}", mode.name(), sec.brief),
                        reason: format!("Mock refinement in {} mode", mode.name()),
                    });
                }
            }

            let cs = RefinementChangeset {
                changes,
                suggested_sections: Vec::new(),
                open_questions: Vec::new(),
                warnings: Vec::new(),
                critique: if mode == RefinementMode::Critique {
                    Some("Document structure looks solid.".to_string())
                } else {
                    None
                },
            };

            cs.validate_against_document(doc)?;
            Ok(cs)
        })
    }
}
