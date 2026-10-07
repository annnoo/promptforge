use crate::document::PromptDocument;
use crate::section::PromptSection;
use serde::{Deserialize, Serialize};

/// A named historical snapshot or branch of a prompt document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: String,
    pub title: String,
    pub sections: Vec<PromptSection>,
}

impl DocumentSnapshot {
    /// Captures the current state of a document as a named snapshot.
    pub fn capture(
        doc: &PromptDocument,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description,
            created_at: chrono_like_timestamp(),
            title: doc.title.clone(),
            sections: doc.sections.clone(),
        }
    }

    /// Restores a prompt document to the state captured in this snapshot.
    pub fn restore_into(&self, doc: &mut PromptDocument) {
        doc.title = self.title.clone();
        doc.sections = self.sections.clone();
    }
}

/// Diff status for comparing two sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionDiffStatus {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Section-level comparison item between historical snapshot and current state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionDiffSummary {
    pub tag: String,
    pub status: SectionDiffStatus,
    pub snapshot_brief: Option<String>,
    pub current_brief: Option<String>,
}

/// Full comparison report between a snapshot and the active document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotComparison {
    pub snapshot_id: String,
    pub snapshot_name: String,
    pub snapshot_title: String,
    pub current_title: String,
    pub is_title_changed: bool,
    pub section_diffs: Vec<SectionDiffSummary>,
}

/// Compares the current document state against a historical snapshot.
pub fn compare_document_with_snapshot(
    current: &PromptDocument,
    snapshot: &DocumentSnapshot,
) -> SnapshotComparison {
    let mut section_diffs = Vec::new();
    let is_title_changed = current.title != snapshot.title;

    // Track matching sections
    let mut matched_snapshot_indices = std::collections::HashSet::new();

    for cur_sec in &current.sections {
        // Try matching by id first, then by tag
        let snap_match = snapshot
            .sections
            .iter()
            .enumerate()
            .find(|(idx, s)| !matched_snapshot_indices.contains(idx) && s.id == cur_sec.id)
            .or_else(|| {
                snapshot
                    .sections
                    .iter()
                    .enumerate()
                    .find(|(idx, s)| !matched_snapshot_indices.contains(idx) && s.tag == cur_sec.tag)
            });

        if let Some((snap_idx, snap_sec)) = snap_match {
            matched_snapshot_indices.insert(snap_idx);
            let is_modified = cur_sec.brief != snap_sec.brief || cur_sec.refined != snap_sec.refined;
            section_diffs.push(SectionDiffSummary {
                tag: cur_sec.tag.clone(),
                status: if is_modified {
                    SectionDiffStatus::Modified
                } else {
                    SectionDiffStatus::Unchanged
                },
                snapshot_brief: Some(snap_sec.brief.clone()),
                current_brief: Some(cur_sec.brief.clone()),
            });
        } else {
            // Added in current
            section_diffs.push(SectionDiffSummary {
                tag: cur_sec.tag.clone(),
                status: SectionDiffStatus::Added,
                snapshot_brief: None,
                current_brief: Some(cur_sec.brief.clone()),
            });
        }
    }

    // Remaining sections in snapshot were removed in current
    for (idx, snap_sec) in snapshot.sections.iter().enumerate() {
        if !matched_snapshot_indices.contains(&idx) {
            section_diffs.push(SectionDiffSummary {
                tag: snap_sec.tag.clone(),
                status: SectionDiffStatus::Removed,
                snapshot_brief: Some(snap_sec.brief.clone()),
                current_brief: None,
            });
        }
    }

    SnapshotComparison {
        snapshot_id: snapshot.id.clone(),
        snapshot_name: snapshot.name.clone(),
        snapshot_title: snapshot.title.clone(),
        current_title: current.title.clone(),
        is_title_changed,
        section_diffs,
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        let days = secs / 86400;
        let rem_secs = secs % 86400;
        let hours = rem_secs / 3600;
        let mins = (rem_secs % 3600) / 60;
        let s = rem_secs % 60;
        format!("Day {} {:02}:{:02}:{:02} UTC", days, hours, mins, s)
    } else {
        "Just now".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_and_restore_snapshot() {
        let mut doc = PromptDocument::new("V1 Title", "Original");
        let s1 = PromptSection::new("role", "Architect").unwrap();
        doc.add_section(s1).unwrap();

        let snap = DocumentSnapshot::capture(&doc, "v1.0 Baseline", Some("First release".into()));
        assert_eq!(snap.name, "v1.0 Baseline");
        assert_eq!(snap.title, "V1 Title");
        assert_eq!(snap.sections.len(), 1);

        // Mutate document
        doc.title = "V2 Title".to_string();
        let s2 = PromptSection::new("task", "New Task").unwrap();
        doc.add_section(s2).unwrap();
        assert_eq!(doc.sections.len(), 2);

        // Compare
        let diff = compare_document_with_snapshot(&doc, &snap);
        assert!(diff.is_title_changed);
        assert_eq!(diff.section_diffs.len(), 2);
        assert_eq!(diff.section_diffs[0].status, SectionDiffStatus::Unchanged);
        assert_eq!(diff.section_diffs[1].status, SectionDiffStatus::Added);

        // Restore
        snap.restore_into(&mut doc);
        assert_eq!(doc.title, "V1 Title");
        assert_eq!(doc.sections.len(), 1);
    }
}
