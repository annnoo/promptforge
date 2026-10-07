use crate::document::PromptDocument;
use crate::error::CoreError;
use crate::xml::{escape_xml_text, RenderOptions};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

/// A named test scenario containing variable values to simulate prompt execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestScenario {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

impl TestScenario {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            variables: HashMap::new(),
        }
    }
}

/// Scans arbitrary text for `{{variable}}` or `{{ variable }}` placeholders.
/// Returns a sorted, deduplicated list of variable names.
pub fn extract_variables_from_text(text: &str) -> Vec<String> {
    let mut vars = BTreeSet::new();
    let mut chars = text.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch == '{' {
            if let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == '{' {
                    chars.next(); // consume second '{'
                    let start = idx + 2;
                    let mut found_end = false;
                    let mut end = start;

                    while let Some((sub_idx, sub_ch)) = chars.next() {
                        if sub_ch == '}' {
                            if let Some(&(_, sub_next)) = chars.peek() {
                                if sub_next == '}' {
                                    chars.next(); // consume second '}'
                                    end = sub_idx;
                                    found_end = true;
                                    break;
                                }
                            }
                        }
                    }

                    if found_end && end > start {
                        let var_name = text[start..end].trim();
                        if !var_name.is_empty() {
                            vars.insert(var_name.to_string());
                        }
                    }
                }
            }
        }
    }

    vars.into_iter().collect()
}

/// Extracts all unique `{{variables}}` referenced across all sections in the document.
pub fn extract_document_variables(doc: &PromptDocument) -> Vec<String> {
    let mut vars = BTreeSet::new();
    for section in &doc.sections {
        for v in extract_variables_from_text(&section.brief) {
            vars.insert(v);
        }
        if let Some(ref refined) = section.refined {
            for v in extract_variables_from_text(refined) {
                vars.insert(v);
            }
        }
    }
    vars.into_iter().collect()
}

/// Substitutes all `{{variable}}` placeholders with values from the given map.
/// If a variable is not found in `values`, it remains unreplaced as `{{variable}}`.
pub fn interpolate_text(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch == '{' {
            if let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == '{' {
                    chars.next(); // consume second '{'
                    let start = idx + 2;
                    let mut found_end = false;
                    let mut end = start;

                    while let Some((sub_idx, sub_ch)) = chars.next() {
                        if sub_ch == '}' {
                            if let Some(&(_, sub_next)) = chars.peek() {
                                if sub_next == '}' {
                                    chars.next(); // consume second '}'
                                    end = sub_idx;
                                    found_end = true;
                                    break;
                                }
                            }
                        }
                    }

                    if found_end && end > start {
                        let var_name = template[start..end].trim();
                        if let Some(replacement) = values.get(var_name) {
                            result.push_str(replacement);
                        } else {
                            // Preserve original placeholder
                            result.push_str(&template[idx..end + 2]);
                        }
                        continue;
                    }
                }
            }
        }
        result.push(ch);
    }

    result
}

/// Renders the prompt document into XML with variable interpolation applied.
pub fn render_interpolated_xml(
    doc: &PromptDocument,
    options: RenderOptions,
    values: &HashMap<String, String>,
) -> Result<String, CoreError> {
    doc.validate()?;

    let mut out = String::new();
    out.push_str("<prompt>\n");

    for section in &doc.sections {
        if !section.enabled {
            continue;
        }

        let raw_content = section.effective_content(options.stage);
        let interpolated = interpolate_text(raw_content, values);
        let escaped_content = escape_xml_text(&interpolated);

        let id_attr = if options.include_ids {
            format!(" id=\"{}\"", section.id)
        } else {
            String::new()
        };

        if options.pretty {
            out.push_str(&format!("  <{}{}>\n", section.tag, id_attr));
            if !escaped_content.is_empty() {
                for line in escaped_content.lines() {
                    out.push_str("    ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
            out.push_str(&format!("  </{}>\n", section.tag));
        } else {
            out.push_str(&format!("<{}{}>", section.tag, id_attr));
            out.push_str(&escaped_content);
            out.push_str(&format!("</{}>", section.tag));
        }
    }

    out.push_str("</prompt>\n");
    Ok(out)
}

/// Estimates token count and computes character/word metrics for a given prompt string.
pub fn calculate_prompt_telemetry(text: &str) -> (usize, usize, usize) {
    let char_count = text.len();
    let word_count = if text.trim().is_empty() {
        0
    } else {
        text.split_whitespace().count()
    };
    // Standard rule of thumb: ~4 characters per token for English & code
    let estimated_tokens = (char_count + 3) / 4;
    (char_count, word_count, estimated_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PromptSection;

    #[test]
    fn test_extract_variables() {
        let text = "Hello {{ user }}, welcome to {{env}}! Check {{ user }} again with {{ timeout_seconds }}.";
        let vars = extract_variables_from_text(text);
        assert_eq!(vars, vec!["env", "timeout_seconds", "user"]);
    }

    #[test]
    fn test_interpolate_text() {
        let text = "Analyze {{repo_path}} using {{lang}} compiler with timeout {{timeout}}.";
        let mut vals = HashMap::new();
        vals.insert("repo_path".to_string(), "/home/src".to_string());
        vals.insert("lang".to_string(), "Rust".to_string());

        let res = interpolate_text(text, &vals);
        // Missing timeout is preserved as {{timeout}}
        assert_eq!(
            res,
            "Analyze /home/src using Rust compiler with timeout {{timeout}}."
        );
    }

    #[test]
    fn test_extract_document_variables() {
        let mut doc = PromptDocument::new("Doc", "Desc");
        let s1 = PromptSection::new("role", "Architect for {{target_platform}}").unwrap();
        let s2 = PromptSection::new("task", "Deploy to {{target_platform}} in {{region}}").unwrap();
        doc.add_section(s1).unwrap();
        doc.add_section(s2).unwrap();

        let vars = extract_document_variables(&doc);
        assert_eq!(vars, vec!["region", "target_platform"]);
    }

    #[test]
    fn test_render_interpolated_xml() {
        let mut doc = PromptDocument::new("Doc", "Desc");
        let s1 = PromptSection::new("role", "Engineer for {{target}}").unwrap();
        doc.add_section(s1).unwrap();

        let mut vals = HashMap::new();
        vals.insert("target".to_string(), "Linux x86_64".to_string());

        let xml = render_interpolated_xml(&doc, RenderOptions::final_clean(), &vals).unwrap();
        assert!(xml.contains("Engineer for Linux x86_64"));
    }
}
