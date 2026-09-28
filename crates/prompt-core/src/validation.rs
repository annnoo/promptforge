use crate::error::CoreError;
use std::collections::HashSet;
use uuid::Uuid;

/// Validates whether a given string is a valid XML element tag name for PromptForge.
///
/// Rules:
/// - Cannot be empty
/// - Cannot start with numbers, hyphens, periods, or colons
/// - First character must be ASCII alphabetic or underscore
/// - Subsequent characters must be ASCII alphanumeric, underscores, hyphens, or periods
/// - Cannot be the reserved prefix/name "xml" (case-insensitive)
pub fn validate_tag_name(tag: &str) -> Result<(), CoreError> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return Err(CoreError::InvalidTagName {
            tag: tag.to_string(),
            reason: "Tag name cannot be empty".to_string(),
        });
    }

    if trimmed.len() != tag.len() {
        return Err(CoreError::InvalidTagName {
            tag: tag.to_string(),
            reason: "Tag name cannot contain leading or trailing whitespace".to_string(),
        });
    }

    if trimmed.to_ascii_lowercase().starts_with("xml") {
        return Err(CoreError::InvalidTagName {
            tag: tag.to_string(),
            reason: "Tag names starting with 'xml' are reserved in XML specification".to_string(),
        });
    }

    let mut chars = trimmed.chars();
    let first = chars.next().unwrap();

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(CoreError::InvalidTagName {
            tag: tag.to_string(),
            reason: format!(
                "First character '{first}' must be an ASCII letter or underscore"
            ),
        });
    }

    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.') {
            return Err(CoreError::InvalidTagName {
                tag: tag.to_string(),
                reason: format!(
                    "Character '{ch}' is not allowed in XML tag name (allowed: letters, digits, _, -, .)"
                ),
            });
        }
    }

    Ok(())
}

/// Validates that a set of section IDs does not contain duplicates.
pub fn validate_unique_section_ids<'a, I>(ids: I) -> Result<(), CoreError>
where
    I: IntoIterator<Item = &'a Uuid>,
{
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(*id) {
            return Err(CoreError::DuplicateSectionId(*id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tag_names() {
        assert!(validate_tag_name("role").is_ok());
        assert!(validate_tag_name("Role").is_ok());
        assert!(validate_tag_name("context_1").is_ok());
        assert!(validate_tag_name("output-format").is_ok());
        assert!(validate_tag_name("section.name").is_ok());
        assert!(validate_tag_name("_private").is_ok());
    }

    #[test]
    fn test_invalid_tag_names() {
        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("  ").is_err());
        assert!(validate_tag_name("1role").is_err());
        assert!(validate_tag_name("-task").is_err());
        assert!(validate_tag_name("task name").is_err());
        assert!(validate_tag_name("task<name>").is_err());
        assert!(validate_tag_name("task/name").is_err());
        assert!(validate_tag_name("xml").is_err());
        assert!(validate_tag_name("XML").is_err());
    }
}
