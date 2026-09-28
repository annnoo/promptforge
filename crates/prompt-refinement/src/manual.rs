use crate::changeset::{RefinementChangeset, SectionChange};
use crate::error::RefinementError;
use crate::prompts::{generate_system_prompt, generate_user_prompt, RefinementMode};
use prompt_core::{parse_xml_refinements_by_id, PromptDocument};

/// Generates a complete text request suitable for copying into any external LLM web UI.
pub fn generate_manual_request(
    doc: &PromptDocument,
    mode: RefinementMode,
) -> Result<String, RefinementError> {
    let system = generate_system_prompt(mode);
    let user = generate_user_prompt(doc);

    Ok(format!(
        "================================================================================\n\
         SYSTEM INSTRUCTIONS\n\
         ================================================================================\n\
         {}\n\n\
         ================================================================================\n\
         USER PROMPT & DOCUMENT\n\
         ================================================================================\n\
         {}\n",
        system, user
    ))
}

/// Parses and validates a response pasted by the user from an external LLM.
/// Supports both JSON changesets and valid refined XML documents with matching IDs.
pub fn parse_manual_response(
    raw: &str,
    doc: &PromptDocument,
) -> Result<RefinementChangeset, RefinementError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(RefinementError::InvalidPayload(
            "Response is empty".to_string(),
        ));
    }

    // Try parsing as JSON first
    if let Ok(changeset) = try_parse_json_changeset(trimmed) {
        changeset.validate_against_document(doc)?;
        return Ok(changeset);
    }

    // Next, try parsing as XML
    if let Ok(changeset) = try_parse_xml_response(trimmed, doc) {
        changeset.validate_against_document(doc)?;
        return Ok(changeset);
    }

    Err(RefinementError::InvalidPayload(
        "Could not parse response as either a valid PromptForge JSON changeset or valid XML document with section IDs".to_string(),
    ))
}

/// Helper to parse JSON either directly or from markdown fences (` ```json `).
fn try_parse_json_changeset(text: &str) -> Result<RefinementChangeset, RefinementError> {
    let json_text = extract_code_block(text, "json").unwrap_or(text);

    // Look for JSON object start '{' and end '}'
    if let Some(start) = json_text.find('{') {
        if let Some(end) = json_text.rfind('}') {
            let slice = &json_text[start..=end];
            let parsed: RefinementChangeset = serde_json::from_str(slice)?;
            return Ok(parsed);
        }
    }

    let parsed: RefinementChangeset = serde_json::from_str(json_text)?;
    Ok(parsed)
}

/// Helper to parse XML either directly or from markdown fences (` ```xml `).
fn try_parse_xml_response(
    text: &str,
    doc: &PromptDocument,
) -> Result<RefinementChangeset, RefinementError> {
    let xml_text = extract_code_block(text, "xml").unwrap_or(text);
    let map = parse_xml_refinements_by_id(xml_text)?;

    if map.is_empty() {
        return Err(RefinementError::InvalidPayload(
            "No sections with matching ID attributes found in XML".to_string(),
        ));
    }

    let mut changes = Vec::new();
    let mut warnings = Vec::new();

    for (id, refined_content) in map {
        if let Some(section) = doc.section(id) {
            if section.locked {
                warnings.push(format!(
                    "Skipped locked section <{}> (id: {})",
                    section.tag, id
                ));
                continue;
            }

            // Only add as a change if it actually differs from current brief
            if section.brief.trim() != refined_content.trim() {
                changes.push(SectionChange {
                    section_id: id,
                    refined: refined_content,
                    reason: "Imported from refined XML".to_string(),
                });
            }
        } else {
            warnings.push(format!("Ignored unknown section ID '{}'", id));
        }
    }

    Ok(RefinementChangeset {
        changes,
        warnings,
        ..Default::default()
    })
}

/// Extracts content from within markdown code blocks ```lang ... ``` if present.
fn extract_code_block<'a>(text: &'a str, lang: &str) -> Option<&'a str> {
    let fence_start = format!("```{}", lang);
    if let Some(start_pos) = text.find(&fence_start) {
        let content_start = start_pos + fence_start.len();
        let rest = &text[content_start..];
        // skip newline
        let rest = rest.strip_prefix("\r\n").or_else(|| rest.strip_prefix('\n')).unwrap_or(rest);
        if let Some(end_pos) = rest.find("```") {
            return Some(&rest[..end_pos]);
        }
    }
    // Also try plain ``` without lang
    if let Some(start_pos) = text.find("```") {
        let content_start = start_pos + 3;
        let rest = &text[content_start..];
        let rest = rest.strip_prefix("\r\n").or_else(|| rest.strip_prefix('\n')).unwrap_or(rest);
        if let Some(end_pos) = rest.find("```") {
            return Some(&rest[..end_pos]);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use prompt_core::PromptSection;

    #[test]
    fn test_parse_json_manual_response() {
        let mut doc = PromptDocument::new("Doc", "");
        let sec = PromptSection::new("role", "Junior dev").unwrap();
        let id = doc.add_section(sec).unwrap();

        let json_payload = format!(
            r#"```json
            {{
              "changes": [
                {{
                  "section_id": "{}",
                  "refined": "Principal Rust Engineer",
                  "reason": "Clarifies expertise"
                }}
              ],
              "suggested_sections": [],
              "open_questions": [],
              "warnings": []
            }}
            ```"#,
            id
        );

        let changeset = parse_manual_response(&json_payload, &doc).unwrap();
        assert_eq!(changeset.changes.len(), 1);
        assert_eq!(changeset.changes[0].section_id, id);
        assert_eq!(changeset.changes[0].refined, "Principal Rust Engineer");
    }

    #[test]
    fn test_parse_xml_manual_response() {
        let mut doc = PromptDocument::new("Doc", "");
        let sec = PromptSection::new("role", "Junior dev").unwrap();
        let id = doc.add_section(sec).unwrap();

        let xml_payload = format!(
            r#"
            <prompt>
              <role id="{}">
                Lead Distributed Systems Architect
              </role>
            </prompt>
            "#,
            id
        );

        let changeset = parse_manual_response(&xml_payload, &doc).unwrap();
        assert_eq!(changeset.changes.len(), 1);
        assert_eq!(changeset.changes[0].section_id, id);
        assert_eq!(changeset.changes[0].refined, "Lead Distributed Systems Architect");
    }
}
