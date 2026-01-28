//! Settings Help Text
//!
//! Loads contextual help text for the Settings screen from embedded assets
//! and user overrides.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Help text entry for a settings item
#[derive(Debug, Clone, Deserialize)]
pub struct SettingsHelpEntry {
    /// Title for the help panel
    pub title: String,
    /// Description text
    pub description: String,
}

/// Store for settings help text
#[derive(Debug, Clone, Default)]
pub struct SettingsHelpStore {
    entries: HashMap<String, SettingsHelpEntry>,
}

impl SettingsHelpStore {
    /// Load help text from embedded assets
    pub fn load_embedded() -> Self {
        let mut store = Self::default();

        let content = include_str!("../../assets/metadata/settings_help.toml");
        if let Ok(entries) = toml::from_str::<HashMap<String, SettingsHelpEntry>>(content) {
            store.entries = entries;
        }

        store
    }

    /// Load user overrides from a file
    pub fn load_user_overrides(&mut self, path: &Path) {
        if !path.exists() {
            return;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(entries) = toml::from_str::<HashMap<String, SettingsHelpEntry>>(&content) {
                // Merge user entries (override embedded)
                for (key, value) in entries {
                    self.entries.insert(key, value);
                }
            }
        }
    }

    /// Get help text for a settings item by key
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&SettingsHelpEntry> {
        self.entries.get(key)
    }

    /// Get help text or return default values
    pub fn get_or_default(&self, key: &str) -> (&str, &str) {
        self.entries
            .get(key)
            .map(|e| (e.title.as_str(), e.description.as_str()))
            .or_else(|| {
                self.entries
                    .get("default")
                    .map(|e| (e.title.as_str(), e.description.as_str()))
            })
            .unwrap_or(("Settings", "Select a setting to see its description."))
    }
}
