use crate::error::PersistenceError;
use prompt_core::{
    render_xml, PromptDocument, RenderOptions, CURRENT_SCHEMA_VERSION,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub const PROMPT_EXTENSION: &str = "prompt.json";

/// Saves a `PromptDocument` atomically to the specified path as pretty JSON.
///
/// It writes first to a temporary sibling file and renames it on success,
/// ensuring that a crash or interrupted write never corrupts an existing document.
pub fn save_document(path: &Path, doc: &PromptDocument) -> Result<(), PersistenceError> {
    doc.validate()?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| PersistenceError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));

    let json_bytes = serde_json::to_vec_pretty(doc)?;

    {
        let mut file = File::create(&temp_path).map_err(|e| PersistenceError::Io {
            path: temp_path.clone(),
            source: e,
        })?;

        file.write_all(&json_bytes).map_err(|e| PersistenceError::Io {
            path: temp_path.clone(),
            source: e,
        })?;

        file.flush().map_err(|e| PersistenceError::Io {
            path: temp_path.clone(),
            source: e,
        })?;

        file.sync_all().map_err(|e| PersistenceError::Io {
            path: temp_path.clone(),
            source: e,
        })?;
    }

    fs::rename(&temp_path, path).map_err(|e| PersistenceError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Loads a `PromptDocument` from a JSON file, checking schema compatibility and validating invariants.
pub fn load_document(path: &Path) -> Result<PromptDocument, PersistenceError> {
    let content = fs::read_to_string(path).map_err(|e| PersistenceError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let doc: PromptDocument =
        serde_json::from_str(&content).map_err(|e| PersistenceError::JsonDeserialization {
            path: path.to_path_buf(),
            source: e,
        })?;

    if doc.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(PersistenceError::UnsupportedSchemaVersion {
            found: doc.schema_version,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }

    doc.validate().map_err(|e| PersistenceError::InvalidDocument {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    Ok(doc)
}

/// Exports a rendered XML prompt document to a target file.
pub fn export_xml(
    path: &Path,
    doc: &PromptDocument,
    options: RenderOptions,
) -> Result<(), PersistenceError> {
    let xml = render_xml(doc, options)?;
    export_text(path, &xml)
}

/// Exports a prompt document as an Agent Skill markdown file.
pub fn export_skill(
    path: &Path,
    doc: &PromptDocument,
    options: prompt_core::SkillExportOptions,
) -> Result<(), PersistenceError> {
    let md = prompt_core::generate_skill_markdown(doc, options)?;
    export_text(path, &md)
}

/// Atomically exports arbitrary text to a target file.
pub fn export_text(path: &Path, text: &str) -> Result<(), PersistenceError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| PersistenceError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
    }

    let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));

    {
        let mut file = File::create(&temp_path).map_err(|e| PersistenceError::Io {
            path: temp_path.clone(),
            source: e,
        })?;

        file.write_all(text.as_bytes())
            .map_err(|e| PersistenceError::Io {
                path: temp_path.clone(),
                source: e,
            })?;

        file.flush().map_err(|e| PersistenceError::Io {
            path: temp_path.clone(),
            source: e,
        })?;
    }

    fs::rename(&temp_path, path).map_err(|e| PersistenceError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Reads text from a file.
pub fn read_text(path: &Path) -> Result<String, PersistenceError> {
    fs::read_to_string(path).map_err(|e| PersistenceError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prompt_core::PromptSection;

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("promptforge_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test.prompt.json");

        let mut doc = PromptDocument::new("Test Persistence", "Checking roundtrip");
        let s1 = PromptSection::new("role", "Tester").unwrap();
        let s2 = PromptSection::new("task", "Verify persistence").unwrap();
        doc.add_section(s1).unwrap();
        doc.add_section(s2).unwrap();

        save_document(&file_path, &doc).unwrap();
        let loaded = load_document(&file_path).unwrap();

        assert_eq!(doc.id, loaded.id);
        assert_eq!(doc.title, loaded.title);
        assert_eq!(doc.sections.len(), loaded.sections.len());
        assert_eq!(doc.sections[0].id, loaded.sections[0].id);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
