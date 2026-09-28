pub mod changeset;
pub mod diff;
pub mod engine;
pub mod error;
pub mod manual;
pub mod openai;
pub mod prompts;
pub mod provider;

pub use changeset::{RefinementChangeset, SectionChange, SuggestedSection};
pub use diff::{compute_line_diff, DiffLine, DiffTag};
pub use engine::RefinementEngine;
pub use error::RefinementError;
pub use manual::{generate_manual_request, parse_manual_response};
pub use openai::OpenAiCompatibleProvider;
pub use prompts::{generate_system_prompt, generate_user_prompt, RefinementMode};
pub use provider::{MockProvider, RefinementProvider};
