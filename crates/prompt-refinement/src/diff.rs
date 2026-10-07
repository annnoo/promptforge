use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

/// Tag indicating the type of change on a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffTag {
    Equal,
    Delete,
    Insert,
}

/// A single line in a diff view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub text: String,
}

/// Computes a line-by-line diff between two strings.
pub fn compute_line_diff(original: &str, refined: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(original, refined);
    let mut lines = Vec::new();

    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Equal => DiffTag::Equal,
            ChangeTag::Delete => DiffTag::Delete,
            ChangeTag::Insert => DiffTag::Insert,
        };
        let cleaned = change
            .value()
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();

        lines.push(DiffLine { tag, text: cleaned });
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff() {
        let old = "Line 1\nLine 2\nLine 3";
        let new = "Line 1\nLine 2 modified\nLine 3";
        let diff = compute_line_diff(old, new);

        assert!(diff.iter().any(|d| d.tag == DiffTag::Delete && d.text == "Line 2"));
        assert!(diff.iter().any(|d| d.tag == DiffTag::Insert && d.text == "Line 2 modified"));
        assert!(diff.iter().any(|d| d.tag == DiffTag::Equal && d.text == "Line 1"));
    }
}
