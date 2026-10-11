use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::atomic::write_atomic;
use super::location::project_dirs;
use super::{SaveLocation, StorageError};

/// App-wide settings that apply before any campaign is opened.
///
/// Always stored in the OS config folder (e.g. `~/Library/Application Support/choir-of-flesh/`
/// on macOS, `~/.config/choir-of-flesh/` on Linux), because it's what tells the app where
/// everything else lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub save_location: SaveLocation,
}

impl AppConfig {
    /// The standard location of the config file.
    pub fn default_path() -> Result<PathBuf, StorageError> {
        Ok(project_dirs()?.config_dir().join("config.json"))
    }

    /// Loads the config, or returns the defaults if it has never been saved.
    pub fn load(path: &Path) -> Result<Self, StorageError> {
        match std::fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).map_err(StorageError::corrupt(path)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(StorageError::io(path)(e)),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), StorageError> {
        let json = serde_json::to_string_pretty(self).expect("AppConfig always serializes");
        write_atomic(path, &json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let root = tempfile::tempdir().unwrap();
        let config = AppConfig::load(&root.path().join("config.json")).unwrap();
        assert_eq!(config.save_location, SaveLocation::System);
    }

    #[test]
    fn round_trip() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        let config = AppConfig {
            save_location: SaveLocation::Local,
        };
        config.save(&path).unwrap();
        assert_eq!(AppConfig::load(&path).unwrap(), config);
    }

    #[test]
    fn garbage_is_reported_not_ignored() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.json");
        std::fs::write(&path, "not json").unwrap();
        assert!(matches!(
            AppConfig::load(&path),
            Err(StorageError::Corrupt { .. })
        ));
    }
}
