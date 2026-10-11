use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::atomic::write_atomic;
use super::{SaveLocation, StorageError};
use crate::campaign::{CURRENT_FORMAT_VERSION, Campaign};

/// The folder of campaign save files: `<id>.json`, plus `<id>.json.bak` holding the
/// previous save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignStore {
    dir: PathBuf,
}

/// What the main menu needs to show for one campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignSummary {
    pub id: String,
    pub name: String,
    pub updated: DateTime<Utc>,
}

/// Every campaign in the folder, most recently played first. Files that couldn't be read
/// are reported separately instead of hiding the rest.
#[derive(Debug, Default)]
pub struct CampaignList {
    pub campaigns: Vec<CampaignSummary>,
    pub unreadable: Vec<StorageError>,
}

/// Just enough of a save to check its version before parsing the rest.
#[derive(Deserialize)]
struct Header {
    format_version: u32,
}

impl CampaignStore {
    /// The store for a [`SaveLocation`].
    pub fn open(location: SaveLocation) -> Result<Self, StorageError> {
        Ok(Self::at(location.campaigns_dir()?))
    }

    /// A store in a specific folder.
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn list(&self) -> Result<CampaignList, StorageError> {
        let mut list = CampaignList::default();
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(list),
            Err(e) => return Err(StorageError::io(&self.dir)(e)),
        };
        for entry in entries {
            let path = entry.map_err(StorageError::io(&self.dir))?.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            match read_campaign(&path) {
                Ok(c) => list.campaigns.push(CampaignSummary {
                    id: c.id,
                    name: c.name,
                    updated: c.updated,
                }),
                Err(e) => list.unreadable.push(e),
            }
        }
        list.campaigns.sort_by_key(|c| std::cmp::Reverse(c.updated));
        Ok(list)
    }

    /// Creates and saves a new campaign with a unique id derived from its name.
    pub fn create(&self, name: &str) -> Result<Campaign, StorageError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(StorageError::EmptyName);
        }
        let base = slug(name);
        let id = std::iter::once(base.clone())
            .chain((2..).map(|n| format!("{base}-{n}")))
            .find(|id| !self.path(id).exists())
            .expect("an unused id always exists");
        let mut campaign = Campaign::new(id, name);
        self.save(&mut campaign)?;
        Ok(campaign)
    }

    pub fn load(&self, id: &str) -> Result<Campaign, StorageError> {
        let path = self.path(id);
        if !path.exists() {
            return Err(StorageError::NotFound(id.to_owned()));
        }
        read_campaign(&path)
    }

    /// Saves the campaign (stamping `updated`), keeping the previous file as a backup.
    pub fn save(&self, campaign: &mut Campaign) -> Result<(), StorageError> {
        campaign.updated = Utc::now();
        let path = self.path(&campaign.id);
        if path.exists() {
            let backup = backup_path(&path);
            fs::copy(&path, &backup).map_err(StorageError::io(backup))?;
        }
        let json = serde_json::to_string_pretty(campaign).expect("Campaign always serializes");
        write_atomic(&path, &json)
    }

    /// Permanently deletes a campaign and its backup.
    pub fn delete(&self, id: &str) -> Result<(), StorageError> {
        let path = self.path(id);
        fs::remove_file(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => StorageError::NotFound(id.to_owned()),
            _ => StorageError::io(&path)(e),
        })?;
        let backup = backup_path(&path);
        if backup.exists() {
            fs::remove_file(&backup).map_err(StorageError::io(backup))?;
        }
        Ok(())
    }

    fn path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }
}

fn read_campaign(path: &Path) -> Result<Campaign, StorageError> {
    let json = fs::read_to_string(path).map_err(StorageError::io(path))?;
    let header: Header = serde_json::from_str(&json).map_err(StorageError::corrupt(path))?;
    if header.format_version > CURRENT_FORMAT_VERSION {
        return Err(StorageError::NewerFormat {
            path: path.to_owned(),
            found: header.format_version,
            supported: CURRENT_FORMAT_VERSION,
        });
    }
    serde_json::from_str(&json).map_err(StorageError::corrupt(path))
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".bak");
    name.into()
}

/// A file-safe id from a name: "Hope's Watch" → "hope-s-watch".
fn slug(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "campaign".to_owned()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests;
