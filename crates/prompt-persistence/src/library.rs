use crate::document_io::{load_document, save_document, PROMPT_EXTENSION};
use crate::error::PersistenceError;
use prompt_core::PromptDocument;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Summary metadata for a saved prompt in the local Prompt Library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPromptSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub section_count: usize,
    pub tags: Vec<String>,
    pub file_path: String,
    pub updated_at: String,
}

/// Lists all saved prompt documents in the specified library directory.
pub fn list_saved_prompts(dir: &Path) -> Result<Vec<SavedPromptSummary>, PersistenceError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| PersistenceError::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.ends_with(PROMPT_EXTENSION) || file_name.ends_with(".json") {
                if let Ok(doc) = load_document(&path) {
                    let mut tag_set = BTreeSet::new();
                    for s in &doc.sections {
                        for t in &s.tags {
                            tag_set.insert(t.clone());
                        }
                    }

                    let updated_at = entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(format_system_time)
                        .unwrap_or_else(|| "Unknown".to_string());

                    summaries.push(SavedPromptSummary {
                        id: doc.id.to_string(),
                        title: doc.title,
                        description: doc.description,
                        section_count: doc.sections.len(),
                        tags: tag_set.into_iter().collect(),
                        file_path: path.to_string_lossy().to_string(),
                        updated_at,
                    });
                }
            }
        }
    }

    // Sort by title alphabetically
    summaries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(summaries)
}

/// Saves a prompt document to the library directory.
pub fn save_to_library(
    dir: &Path,
    doc: &PromptDocument,
) -> Result<SavedPromptSummary, PersistenceError> {
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| PersistenceError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
    }

    let file_path = dir.join(format!("{}.{}", doc.id, PROMPT_EXTENSION));
    save_document(&file_path, doc)?;

    let mut tag_set = BTreeSet::new();
    for s in &doc.sections {
        for t in &s.tags {
            tag_set.insert(t.clone());
        }
    }

    Ok(SavedPromptSummary {
        id: doc.id.to_string(),
        title: doc.title.clone(),
        description: doc.description.clone(),
        section_count: doc.sections.len(),
        tags: tag_set.into_iter().collect(),
        file_path: file_path.to_string_lossy().to_string(),
        updated_at: format_system_time(SystemTime::now()),
    })
}

/// Loads a prompt document from the library by ID or file path.
pub fn load_from_library(dir: &Path, id: &str) -> Result<PromptDocument, PersistenceError> {
    let candidate = dir.join(format!("{}.{}", id, PROMPT_EXTENSION));
    if candidate.exists() {
        return load_document(&candidate);
    }

    // Also search by filename or document id
    let entries = fs::read_dir(dir).map_err(|e| PersistenceError::Io {
        path: dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(doc) = load_document(&path) {
                if doc.id.to_string() == id || doc.title.eq_ignore_ascii_case(id) {
                    return Ok(doc);
                }
            }
        }
    }

    Err(PersistenceError::InvalidDocument {
        path: PathBuf::from(id),
        reason: format!("Prompt with ID '{}' not found in library", id),
    })
}

/// Deletes a prompt document from the library by ID.
pub fn delete_from_library(dir: &Path, id: &str) -> Result<bool, PersistenceError> {
    let candidate = dir.join(format!("{}.{}", id, PROMPT_EXTENSION));
    if candidate.exists() {
        fs::remove_file(&candidate).map_err(|e| PersistenceError::Io {
            path: candidate,
            source: e,
        })?;
        return Ok(true);
    }

    // Search by document ID inside directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(doc) = load_document(&path) {
                    if doc.id.to_string() == id {
                        fs::remove_file(&path).map_err(|e| PersistenceError::Io {
                            path,
                            source: e,
                        })?;
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

fn format_system_time(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        // Return a clean timestamp string
        let days = secs / 86400;
        let rem_secs = secs % 86400;
        let hours = rem_secs / 3600;
        let mins = (rem_secs % 3600) / 60;
        // Basic readable date approximation without pulling heavy chrono
        format!("Day {} {:02}:{:02} UTC", days, hours, mins)
    } else {
        "Recent".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prompt_core::PromptSection;

    #[test]
    fn test_library_save_list_load_delete() {
        let temp_dir = std::env::temp_dir().join(format!("promptforge_lib_test_{}", uuid::Uuid::new_v4()));
        let mut doc = PromptDocument::new("Library Test Doc", "Prompt for library testing");
        let mut s = PromptSection::new("role", "Tester").unwrap();
        s.tags = vec!["qa".to_string(), "testing".to_string()];
        doc.add_section(s).unwrap();

        // 1. Save to library
        let saved = save_to_library(&temp_dir, &doc).unwrap();
        assert_eq!(saved.title, "Library Test Doc");
        assert_eq!(saved.section_count, 1);
        assert_eq!(saved.tags, vec!["qa".to_string(), "testing".to_string()]);

        // 2. List library
        let list = list_saved_prompts(&temp_dir).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, doc.id.to_string());

        // 3. Load from library
        let loaded = load_from_library(&temp_dir, &doc.id.to_string()).unwrap();
        assert_eq!(loaded.id, doc.id);
        assert_eq!(loaded.title, doc.title);

        // 4. Delete from library
        let deleted = delete_from_library(&temp_dir, &doc.id.to_string()).unwrap();
        assert!(deleted);
        let list_after = list_saved_prompts(&temp_dir).unwrap();
        assert_eq!(list_after.len(), 0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
