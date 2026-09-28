pub mod config;
pub mod document_io;
pub mod error;

pub use config::{AppConfig, OpenAiCompatibleConfig, ProviderType};
pub use document_io::{
    export_text, export_xml, load_document, read_text, save_document, PROMPT_EXTENSION,
};
pub use error::PersistenceError;
