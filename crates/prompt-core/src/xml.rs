use crate::document::PromptDocument;
use crate::error::CoreError;
use crate::section::RenderStage;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration options for XML rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    /// Render stage: Draft (original briefs) or Final (accepted refined content if present).
    pub stage: RenderStage,
    /// Whether to include internal section UUIDs as XML attributes (`id="..."`).
    pub include_ids: bool,
    /// Whether to pretty-print with indentation.
    pub pretty: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            stage: RenderStage::Draft,
            include_ids: true,
            pretty: true,
        }
    }
}

impl RenderOptions {
    pub fn draft() -> Self {
        Self {
            stage: RenderStage::Draft,
            include_ids: true,
            pretty: true,
        }
    }

    pub fn final_clean() -> Self {
        Self {
            stage: RenderStage::Final,
            include_ids: false,
            pretty: true,
        }
    }

    pub fn final_with_ids() -> Self {
        Self {
            stage: RenderStage::Final,
            include_ids: true,
            pretty: true,
        }
    }
}

/// Renders a PromptDocument to XML according to the given options.
pub fn render_xml(doc: &PromptDocument, options: RenderOptions) -> Result<String, CoreError> {
    doc.validate()?;

    let mut out = String::new();
    out.push_str("<prompt>\n");

    for section in &doc.sections {
        if !section.enabled {
            continue;
        }

        let content = section.effective_content(options.stage);
        let escaped_content = escape_xml_text(content);

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

/// Escapes standard XML characters in text content.
pub fn escape_xml_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Unescapes standard XML entities in text content.
pub fn unescape_xml_text(text: &str) -> String {
    let mut unescaped = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            let mut found_semicolon = false;
            for next_ch in chars.by_ref() {
                if next_ch == ';' {
                    found_semicolon = true;
                    break;
                }
                entity.push(next_ch);
                if entity.len() > 8 {
                    break;
                }
            }

            if found_semicolon {
                match entity.as_str() {
                    "amp" => unescaped.push('&'),
                    "lt" => unescaped.push('<'),
                    "gt" => unescaped.push('>'),
                    "quot" => unescaped.push('"'),
                    "apos" => unescaped.push('\''),
                    _ => {
                        unescaped.push('&');
                        unescaped.push_str(&entity);
                        unescaped.push(';');
                    }
                }
            } else {
                unescaped.push('&');
                unescaped.push_str(&entity);
            }
        } else {
            unescaped.push(ch);
        }
    }

    unescaped
}

/// Parsed section from an XML document mapping an ID to content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedXmlSection {
    pub id: Option<Uuid>,
    pub tag: String,
    pub content: String,
}

/// Parses an XML prompt document into sections using quick-xml.
/// Extracts any section with an `id` attribute.
pub fn parse_xml_sections(xml_str: &str) -> Result<Vec<ParsedXmlSection>, CoreError> {
    let mut reader = Reader::from_str(xml_str);
    reader.config_mut().trim_text(false);

    let mut sections = Vec::new();
    let mut current_tag: Option<String> = None;
    let mut current_id: Option<Uuid> = None;
    let mut current_text = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name.eq_ignore_ascii_case("prompt") {
                    continue;
                }

                current_tag = Some(tag_name);
                current_id = None;
                current_text.clear();

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref());
                    if key == "id" {
                        let val = String::from_utf8_lossy(&attr.value);
                        if let Ok(parsed_uuid) = Uuid::parse_str(&val) {
                            current_id = Some(parsed_uuid);
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if current_tag.is_some() {
                    let text = e.unescape().map_err(|err| {
                        CoreError::XmlParsingError(format!("Failed to unescape text: {err}"))
                    })?;
                    current_text.push_str(&text);
                }
            }
            Ok(Event::CData(ref e)) => {
                if current_tag.is_some() {
                    let text = String::from_utf8_lossy(e.as_ref());
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if let Some(opened) = current_tag.take() {
                    if opened == tag_name {
                        // Trim uniform indentation while preserving meaningful lines
                        let trimmed_content = trim_xml_indentation(&current_text);
                        sections.push(ParsedXmlSection {
                            id: current_id,
                            tag: opened,
                            content: trimmed_content,
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(CoreError::XmlParsingError(format!(
                    "XML error at position {}: {e:?}",
                    reader.buffer_position()
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(sections)
}

/// Helper to trim leading/trailing common indentation from multiline XML text.
fn trim_xml_indentation(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Skip leading and trailing blank lines
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
    let end = lines.iter().rposition(|l| !l.trim().is_empty()).map(|idx| idx + 1).unwrap_or(0);

    if start >= end {
        return String::new();
    }

    let active_lines = &lines[start..end];
    // Find min indentation among non-empty lines
    let min_indent = active_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    let mut result = Vec::new();
    for line in active_lines {
        if line.trim().is_empty() {
            result.push("");
        } else if line.len() >= min_indent {
            result.push(&line[min_indent..]);
        } else {
            result.push(line.trim_start());
        }
    }

    result.join("\n")
}

/// Parses an XML document and returns a map of UUID -> refined content.
pub fn parse_xml_refinements_by_id(xml_str: &str) -> Result<HashMap<Uuid, String>, CoreError> {
    let sections = parse_xml_sections(xml_str)?;
    let mut map = HashMap::new();
    for sec in sections {
        if let Some(id) = sec.id {
            map.insert(id, sec.content);
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::section::PromptSection;

    #[test]
    fn test_render_xml_deterministic() {
        let mut doc = PromptDocument::new("Test Prompt", "Test Description");
        let s1 = PromptSection::new("role", "Senior backend engineer").unwrap();
        let s2 = PromptSection::new("task", "Build a microservice & test it").unwrap();
        let s3 = PromptSection::new("notes", "Omit this").unwrap();

        let id1 = doc.add_section(s1).unwrap();
        let _id2 = doc.add_section(s2).unwrap();
        let id3 = doc.add_section(s3).unwrap();

        doc.section_mut(id3).unwrap().enabled = false;

        let xml = render_xml(&doc, RenderOptions::draft()).unwrap();
        assert!(xml.contains("<prompt>"));
        assert!(xml.contains(&format!("<role id=\"{}\">", id1)));
        assert!(xml.contains("Senior backend engineer"));
        assert!(xml.contains("Build a microservice &amp; test it"));
        assert!(!xml.contains("Omit this")); // disabled section is omitted
        assert!(xml.contains("</prompt>"));
    }

    #[test]
    fn test_render_xml_clean_without_ids() {
        let mut doc = PromptDocument::new("Clean Prompt", "");
        let s1 = PromptSection::new("role", "Staff Architect").unwrap();
        doc.add_section(s1).unwrap();

        let xml = render_xml(&doc, RenderOptions::final_clean()).unwrap();
        assert!(xml.contains("<role>"));
        assert!(!xml.contains("id="));
    }

    #[test]
    fn test_xml_escaping_and_unescaping() {
        let raw = "Use <T> where T: Clone & 'static with \"quotes\"";
        let escaped = escape_xml_text(raw);
        assert_eq!(
            escaped,
            "Use &lt;T&gt; where T: Clone &amp; &apos;static with &quot;quotes&quot;"
        );
        let unescaped = unescape_xml_text(&escaped);
        assert_eq!(unescaped, raw);
    }

    #[test]
    fn test_parse_xml_sections_roundtrip() {
        let mut doc = PromptDocument::new("Roundtrip", "");
        let s1 = PromptSection::new("role", "Rust developer").unwrap();
        let id1 = doc.add_section(s1).unwrap();

        let rendered = render_xml(&doc, RenderOptions::draft()).unwrap();
        let parsed = parse_xml_sections(&rendered).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, Some(id1));
        assert_eq!(parsed[0].tag, "role");
        assert_eq!(parsed[0].content, "Rust developer");
    }
}
