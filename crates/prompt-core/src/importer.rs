use crate::document::PromptDocument;
use crate::error::CoreError;
use crate::section::PromptSection;
use crate::xml::parse_xml_sections;
use serde::{Deserialize, Serialize};

/// Supported source formats for importing prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    Json,
    Xml,
    Skill,
    Markdown,
    PlainText,
}

/// Result of importing and parsing prompt content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedPrompt {
    pub format: ImportFormat,
    pub title: String,
    pub description: String,
    pub sections: Vec<PromptSection>,
}

impl ImportedPrompt {
    /// Converts this imported prompt into a new `PromptDocument`.
    pub fn to_document(&self) -> Result<PromptDocument, CoreError> {
        let mut doc = PromptDocument::new(&self.title, &self.description);
        for sec in &self.sections {
            doc.add_section(sec.clone())?;
        }
        Ok(doc)
    }
}

/// Detects the format of raw input string.
pub fn detect_import_format(input: &str) -> ImportFormat {
    let trimmed = input.trim();
    if (trimmed.starts_with('{') || trimmed.starts_with("{\n"))
        && (trimmed.contains("\"schema_version\"") || trimmed.contains("\"sections\""))
    {
        return ImportFormat::Json;
    }

    if trimmed.starts_with("---") {
        return ImportFormat::Skill;
    }

    if trimmed.starts_with("<prompt")
        || trimmed.starts_with("<?xml")
        || (trimmed.starts_with('<') && trimmed.contains("</"))
    {
        return ImportFormat::Xml;
    }

    if trimmed.lines().any(|l| l.trim_start().starts_with('#')) {
        return ImportFormat::Markdown;
    }

    ImportFormat::PlainText
}

/// Converts a human-readable heading or name into a valid XML tag name.
pub fn heading_to_tag_name(heading: &str) -> String {
    let mut tag = String::new();
    let mut prev_underscore = false;

    for ch in heading.chars() {
        if ch.is_ascii_alphanumeric() {
            tag.push(ch.to_ascii_lowercase());
            prev_underscore = false;
        } else if (ch == ' ' || ch == '-' || ch == '_') && !prev_underscore && !tag.is_empty() {
            tag.push('_');
            prev_underscore = true;
        }
    }

    let trimmed = tag.trim_matches('_');
    if trimmed.is_empty() {
        "section".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Imports and parses prompt content from any supported text representation.
pub fn import_prompt_from_text(input: &str) -> Result<ImportedPrompt, CoreError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Validation(
            "Import content cannot be empty".to_string(),
        ));
    }

    let format = detect_import_format(trimmed);
    match format {
        ImportFormat::Json => parse_json_import(trimmed),
        ImportFormat::Skill => parse_skill_import(trimmed),
        ImportFormat::Xml => parse_xml_import(trimmed),
        ImportFormat::Markdown => parse_markdown_import(trimmed),
        ImportFormat::PlainText => parse_plain_text_import(trimmed),
    }
}

fn parse_json_import(input: &str) -> Result<ImportedPrompt, CoreError> {
    let doc: PromptDocument = serde_json::from_str(input).map_err(|e| {
        CoreError::Validation(format!("Invalid PromptForge JSON format: {e}"))
    })?;
    doc.validate()?;

    Ok(ImportedPrompt {
        format: ImportFormat::Json,
        title: doc.title,
        description: doc.description,
        sections: doc.sections,
    })
}

fn parse_xml_import(input: &str) -> Result<ImportedPrompt, CoreError> {
    let xml_to_parse = if !input.contains("<prompt") {
        format!("<prompt>\n{}\n</prompt>", input)
    } else {
        input.to_string()
    };

    let parsed_sections = parse_xml_sections(&xml_to_parse)?;
    if parsed_sections.is_empty() {
        return Err(CoreError::Validation(
            "No XML prompt sections found in input".to_string(),
        ));
    }

    let mut sections = Vec::with_capacity(parsed_sections.len());
    for s in parsed_sections {
        let section = match s.id {
            Some(uuid) => PromptSection::with_id(uuid, s.tag, s.content)?,
            None => PromptSection::new(s.tag, s.content)?,
        };
        sections.push(section);
    }

    Ok(ImportedPrompt {
        format: ImportFormat::Xml,
        title: "Imported XML Prompt".to_string(),
        description: format!("Imported from XML with {} sections", sections.len()),
        sections,
    })
}

fn parse_skill_import(input: &str) -> Result<ImportedPrompt, CoreError> {
    // Parse YAML frontmatter between --- and ---
    let mut title = "Imported Skill".to_string();
    let mut description = String::new();
    let mut body = input;

    if input.starts_with("---") {
        let rest = &input[3..];
        if let Some(end_idx) = rest.find("\n---") {
            let frontmatter = &rest[..end_idx];
            body = &rest[end_idx + 4..];

            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("name:") {
                    let cleaned = val.trim().replace('-', " ");
                    // capitalize words
                    title = cleaned
                        .split_whitespace()
                        .map(|w| {
                            let mut c = w.chars();
                            match c.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().chain(c).collect(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                } else if let Some(val) = line.strip_prefix("description:") {
                    description = val.trim().to_string();
                }
            }
        }
    }

    // Try finding an embedded XML code block first
    if let Some(start_xml) = body.find("```xml") {
        let after_start = &body[start_xml + 6..];
        if let Some(end_xml) = after_start.find("```") {
            let xml_block = &after_start[..end_xml];
            if let Ok(mut imported_xml) = parse_xml_import(xml_block) {
                imported_xml.format = ImportFormat::Skill;
                imported_xml.title = title;
                if !description.is_empty() {
                    imported_xml.description = description;
                }
                return Ok(imported_xml);
            }
        }
    }

    // Otherwise parse markdown headers in the body
    let mut parsed_md = parse_markdown_import(body)?;
    parsed_md.format = ImportFormat::Skill;
    if !title.is_empty() {
        parsed_md.title = title;
    }
    if !description.is_empty() {
        parsed_md.description = description;
    }

    Ok(parsed_md)
}

fn parse_markdown_import(input: &str) -> Result<ImportedPrompt, CoreError> {
    let mut title = "Imported Markdown Prompt".to_string();
    let mut description = String::new();
    let mut sections = Vec::new();

    let mut current_tag: Option<String> = None;
    let mut current_lines = Vec::new();

    for line in input.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with('#') {
            // Flush current section if any
            if let Some(tag) = current_tag.take() {
                let content = current_lines.join("\n").trim().to_string();
                if !content.is_empty() {
                    sections.push(PromptSection::new(tag, content)?);
                }
                current_lines.clear();
            }

            let heading_text = trimmed_line.trim_start_matches('#').trim();

            // If it's a top-level H1 and we haven't found any sections yet, treat as title
            if trimmed_line.starts_with("# ") && !trimmed_line.starts_with("## ") && sections.is_empty() && current_tag.is_none() {
                title = heading_text.to_string();
            } else {
                let tag = heading_to_tag_name(heading_text);
                current_tag = Some(tag);
            }
        } else if current_tag.is_some() {
            current_lines.push(line);
        } else if sections.is_empty() && !trimmed_line.is_empty() {
            // Lines before the first section heading belong to description
            if !description.is_empty() {
                description.push('\n');
            }
            description.push_str(trimmed_line);
        }
    }

    if let Some(tag) = current_tag {
        let content = current_lines.join("\n").trim().to_string();
        if !content.is_empty() {
            sections.push(PromptSection::new(tag, content)?);
        }
    }

    if sections.is_empty() {
        // Fallback: create a single task section
        sections.push(PromptSection::new("task", input.trim())?);
    }

    Ok(ImportedPrompt {
        format: ImportFormat::Markdown,
        title,
        description,
        sections,
    })
}

fn parse_plain_text_import(input: &str) -> Result<ImportedPrompt, CoreError> {
    let section = PromptSection::new("task", input.trim())?;
    Ok(ImportedPrompt {
        format: ImportFormat::PlainText,
        title: "Imported Prompt".to_string(),
        description: "Imported from plain text".to_string(),
        sections: vec![section],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_to_tag_name() {
        assert_eq!(heading_to_tag_name("Role"), "role");
        assert_eq!(heading_to_tag_name("Context & Background"), "context_background");
        assert_eq!(heading_to_tag_name("Non-Goals"), "non_goals");
        assert_eq!(heading_to_tag_name("  Acceptance Criteria  "), "acceptance_criteria");
    }

    #[test]
    fn test_import_from_xml() {
        let xml = "<prompt>\n  <role>Architect</role>\n  <task>Build UI</task>\n</prompt>";
        let imported = import_prompt_from_text(xml).unwrap();
        assert_eq!(imported.format, ImportFormat::Xml);
        assert_eq!(imported.sections.len(), 2);
        assert_eq!(imported.sections[0].tag, "role");
        assert_eq!(imported.sections[0].brief, "Architect");
        assert_eq!(imported.sections[1].tag, "task");
        assert_eq!(imported.sections[1].brief, "Build UI");
    }

    #[test]
    fn test_import_from_markdown() {
        let md = "# System Design Prompt\nOverview of the design.\n\n## Role\nPrincipal Engineer\n\n## Task\nDesign DB schema\n\n## Constraints\nNo raw SQL";
        let imported = import_prompt_from_text(md).unwrap();
        assert_eq!(imported.format, ImportFormat::Markdown);
        assert_eq!(imported.title, "System Design Prompt");
        assert_eq!(imported.sections.len(), 3);
        assert_eq!(imported.sections[0].tag, "role");
        assert_eq!(imported.sections[0].brief, "Principal Engineer");
        assert_eq!(imported.sections[1].tag, "task");
        assert_eq!(imported.sections[2].tag, "constraints");
    }

    #[test]
    fn test_import_from_skill() {
        let skill = "---\nname: code-reviewer\ndescription: Reviews code thoroughly\n---\n\n# Code Reviewer\n\n```xml\n<prompt>\n  <role>Senior Reviewer</role>\n  <constraints>Be kind</constraints>\n</prompt>\n```\n";
        let imported = import_prompt_from_text(skill).unwrap();
        assert_eq!(imported.format, ImportFormat::Skill);
        assert_eq!(imported.title, "Code Reviewer");
        assert_eq!(imported.description, "Reviews code thoroughly");
        assert_eq!(imported.sections.len(), 2);
        assert_eq!(imported.sections[0].tag, "role");
        assert_eq!(imported.sections[1].tag, "constraints");
    }
}
