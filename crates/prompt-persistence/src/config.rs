use crate::error::PersistenceError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Provider type selection for refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    #[default]
    Manual,
    OpenAiCompatible,
}

/// Settings for OpenAI-compatible HTTP providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiCompatibleConfig {
    pub base_url: String,
    pub model: String,
    pub api_key_env_var: String,
    pub timeout_seconds: u64,
}

impl Default for OpenAiCompatibleConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
            api_key_env_var: "OPENAI_API_KEY".to_string(),
            timeout_seconds: 60,
        }
    }
}

use prompt_core::{get_builtin_section_types, SectionType};

use std::collections::HashMap;

/// Persistent user configuration for PromptForge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_provider: ProviderType,
    pub openai_compatible: OpenAiCompatibleConfig,
    pub recent_files: Vec<PathBuf>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub custom_types: Vec<SectionType>,
    #[serde(default)]
    pub tag_colors: HashMap<String, String>,
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_provider: ProviderType::Manual,
            openai_compatible: OpenAiCompatibleConfig::default(),
            recent_files: Vec::new(),
            theme: default_theme(),
            custom_types: Vec::new(),
            tag_colors: HashMap::new(),
        }
    }
}

impl AppConfig {
    /// Returns the standard config directory for PromptForge (`~/.config/promptforge`).
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "promptforge", "PromptForge")
            .map(|dirs| dirs.config_dir().to_path_buf())
    }

    /// Returns the standard path to `config.json`.
    pub fn config_file_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.json"))
    }

    /// Loads application configuration from standard config directory, or defaults if not present.
    pub fn load_or_default() -> Self {
        if let Some(path) = Self::config_file_path() {
            if path.exists() {
                if let Ok(config) = Self::load_from(&path) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Loads configuration from a specific file path.
    pub fn load_from(path: &Path) -> Result<Self, PersistenceError> {
        let content = fs::read_to_string(path).map_err(|e| PersistenceError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        serde_json::from_str(&content).map_err(|e| PersistenceError::JsonDeserialization {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Saves configuration to the standard config location.
    pub fn save(&self) -> Result<(), PersistenceError> {
        let path = Self::config_file_path().ok_or_else(|| {
            PersistenceError::Config("Could not determine standard configuration directory".into())
        })?;
        self.save_to(&path)
    }

    /// Saves configuration to a specific file path.
    pub fn save_to(&self, path: &Path) -> Result<(), PersistenceError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PersistenceError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data).map_err(|e| PersistenceError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Records an opened/saved file in the recent files list, capping at 10 entries.
    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    /// Returns all section types: built-ins plus user-created custom types.
    pub fn all_section_types(&self) -> Vec<SectionType> {
        let mut types = get_builtin_section_types();
        types.extend(self.custom_types.clone());
        types
    }

    /// Adds or updates a custom section type by its ID.
    pub fn save_custom_type(&mut self, custom_type: SectionType) {
        if let Some(pos) = self.custom_types.iter().position(|t| t.id == custom_type.id) {
            self.custom_types[pos] = custom_type;
        } else {
            self.custom_types.push(custom_type);
        }
    }

    /// Deletes a custom section type by ID.
    pub fn delete_custom_type(&mut self, id: &str) -> bool {
        if let Some(pos) = self.custom_types.iter().position(|t| t.id == id) {
            self.custom_types.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns the standard directory for saved library prompts (`~/.config/promptforge/library`).
    pub fn library_dir() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("library"))
    }

    /// Sets a customized color for a tag name.
    pub fn set_tag_color(&mut self, tag: String, color: String) {
        self.tag_colors.insert(tag, color);
    }

    /// Removes a custom color mapping for a tag name.
    pub fn remove_tag_color(&mut self, tag: &str) {
        self.tag_colors.remove(tag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_types_management() {
        let mut config = AppConfig::default();
        let initial_count = config.all_section_types().len();
        assert_eq!(initial_count, 13); // 13 built-ins

        let custom = SectionType {
            id: "custom-security".to_string(),
            name: "Security Invariants".to_string(),
            tag: "security_invariants".to_string(),
            description: "Strict safety boundaries".to_string(),
            default_brief: "No secrets in logs.".to_string(),
            folder: "Security".to_string(),
            tags: vec!["safety".to_string(), "prod".to_string()],
            is_builtin: false,
        };

        config.save_custom_type(custom.clone());
        assert_eq!(config.all_section_types().len(), initial_count + 1);

        // Update
        let mut updated = custom.clone();
        updated.description = "Updated description".to_string();
        config.save_custom_type(updated);
        assert_eq!(config.all_section_types().len(), initial_count + 1);

        // Delete
        assert!(config.delete_custom_type("custom-security"));
        assert_eq!(config.all_section_types().len(), initial_count);
    }
}

