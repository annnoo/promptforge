use crate::document::PromptDocument;
use crate::error::CoreError;
use crate::section::RenderStage;
use crate::xml::{render_xml, RenderOptions};

/// Options for exporting a prompt document as an Agent Skill (`SKILL.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillExportOptions {
    /// Stage to render (Draft or Final).
    pub stage: RenderStage,
    /// Custom skill name override (defaults to slugified document title).
    pub custom_name: Option<String>,
    /// Custom skill description override (defaults to document description or title).
    pub custom_description: Option<String>,
}

impl Default for SkillExportOptions {
    fn default() -> Self {
        Self {
            stage: RenderStage::Final,
            custom_name: None,
            custom_description: None,
        }
    }
}

/// Transforms a string into a clean kebab-case name suitable for skill metadata.
pub fn slugify_skill_name(title: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;

    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }

    let trimmed = slug.trim_end_matches('-');
    if trimmed.is_empty() {
        "custom-skill".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generates a standardized Agent Skill markdown document (`SKILL.md`) with YAML frontmatter.
pub fn generate_skill_markdown(
    doc: &PromptDocument,
    options: SkillExportOptions,
) -> Result<String, CoreError> {
    doc.validate()?;

    let name = match options.custom_name.as_deref().map(str::trim) {
        Some(custom) if !custom.is_empty() => slugify_skill_name(custom),
        _ => slugify_skill_name(&doc.title),
    };

    let description = match options.custom_description.as_deref().map(str::trim) {
        Some(custom) if !custom.is_empty() => custom.to_string(),
        _ if !doc.description.trim().is_empty() => doc.description.trim().to_string(),
        _ => format!("Standardized agent prompt skill for {}", doc.title),
    };

    let xml_options = RenderOptions {
        stage: options.stage,
        include_ids: false,
        pretty: true,
    };
    let rendered_xml = render_xml(doc, xml_options)?;

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", name));
    out.push_str(&format!("description: {}\n", description.replace('\n', " ")));
    out.push_str("---\n\n");

    out.push_str(&format!("# {}\n\n", doc.title));

    if !doc.description.trim().is_empty() {
        out.push_str(doc.description.trim());
        out.push_str("\n\n");
    }

    out.push_str("## Prompt Instructions\n\n");
    out.push_str("Execute the following structured prompt directives:\n\n");
    out.push_str("```xml\n");
    out.push_str(&rendered_xml);
    out.push_str("```\n");

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PromptSection;

    #[test]
    fn test_slugify_skill_name() {
        assert_eq!(slugify_skill_name("Code Reviewer"), "code-reviewer");
        assert_eq!(slugify_skill_name("API Architect 2.0!"), "api-architect-2-0");
        assert_eq!(slugify_skill_name("   "), "custom-skill");
        assert_eq!(slugify_skill_name("---hello---world---"), "hello-world");
    }

    #[test]
    fn test_generate_skill_markdown() {
        let mut doc = PromptDocument::new("Rust System Architect", "Design modular systems.");
        let s1 = PromptSection::new("role", "Principal Systems Engineer").unwrap();
        let s2 = PromptSection::new("task", "Architect high-throughput message bus").unwrap();
        doc.add_section(s1).unwrap();
        doc.add_section(s2).unwrap();

        let md = generate_skill_markdown(&doc, SkillExportOptions::default()).unwrap();
        assert!(md.starts_with("---\nname: rust-system-architect\n"));
        assert!(md.contains("description: Design modular systems."));
        assert!(md.contains("# Rust System Architect"));
        assert!(md.contains("<role>"));
        assert!(md.contains("<task>"));
        assert!(md.contains("Principal Systems Engineer"));
    }
}
