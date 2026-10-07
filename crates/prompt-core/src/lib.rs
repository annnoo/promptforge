pub mod document;
pub mod error;
pub mod section;
pub mod template;
pub mod validation;
pub mod xml;

pub use document::{PromptDocument, CURRENT_SCHEMA_VERSION};
pub use error::CoreError;
pub use section::{PromptSection, RenderStage};
pub use template::{
    get_builtin_section_types, SectionPreset, SectionType, StarterTemplate, ALL_PRESETS,
    ALL_TEMPLATES, TEMPLATE_ENGINEERING, TEMPLATE_GENERAL, TEMPLATE_RESEARCH,
};
pub use validation::{validate_tag_name, validate_unique_section_ids};
pub use xml::{
    escape_xml_text, parse_xml_refinements_by_id, parse_xml_sections, render_xml,
    unescape_xml_text, ParsedXmlSection, RenderOptions,
};
