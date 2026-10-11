use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::StorageError;

/// Where campaign files live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SaveLocation {
    /// The operating system's data folder, e.g.
    /// `~/Library/Application Support/choir-of-flesh/campaigns` on macOS or
    /// `~/.local/share/choir-of-flesh/campaigns` on Linux. The same saves wherever you launch.
    #[default]
    System,
    /// A `saves` folder in the directory the toolbox is run from. Opt-in only.
    Local,
}

impl SaveLocation {
    pub fn name(self) -> &'static str {
        match self {
            SaveLocation::System => "System data folder",
            SaveLocation::Local => "Local ./saves folder",
        }
    }

    /// The folder campaign files are stored in.
    pub fn campaigns_dir(self) -> Result<PathBuf, StorageError> {
        match self {
            SaveLocation::System => Ok(project_dirs()?.data_dir().join("campaigns")),
            SaveLocation::Local => {
                let here = std::env::current_dir().map_err(StorageError::io("."))?;
                Ok(here.join("saves"))
            }
        }
    }
}

pub(super) fn project_dirs() -> Result<ProjectDirs, StorageError> {
    ProjectDirs::from("", "", "choir-of-flesh").ok_or(StorageError::NoHomeDirectory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_is_the_default() {
        assert_eq!(SaveLocation::default(), SaveLocation::System);
    }

    #[test]
    fn local_is_saves_in_the_working_directory() {
        let dir = SaveLocation::Local.campaigns_dir().unwrap();
        assert_eq!(dir, std::env::current_dir().unwrap().join("saves"));
    }

    #[test]
    fn system_folder_is_named_for_the_app() {
        let dir = SaveLocation::System.campaigns_dir().unwrap();
        assert!(dir.ends_with("campaigns"));
        assert!(dir.to_string_lossy().contains("choir-of-flesh"));
    }
}
