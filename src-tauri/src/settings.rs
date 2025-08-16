use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: String,
    pub font: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: "light".into(),
            font: "GurbaniAkharThick".into(),
        }
    }
}

pub fn load_settings(path: &PathBuf) -> UserSettings {
    if path.exists() {
        let data = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        UserSettings::default()
    }
}

pub fn save_settings(path: &PathBuf, settings: &UserSettings) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let json = serde_json::to_string_pretty(settings).unwrap();
    fs::write(path, json).unwrap();
}
