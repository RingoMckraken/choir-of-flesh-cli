use std::path::PathBuf;

use cof_core::campaign::Campaign;
use cof_core::character::Character;
use cof_core::storage::{AppConfig, CampaignList, CampaignStore, SaveLocation, StorageError};

/// A one-line message shown at the bottom of the screen until the next key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Info(String),
    Error(String),
}

/// Everything the screens share: app config, the campaign store, and the open campaign.
pub struct AppContext {
    pub config: AppConfig,
    config_path: PathBuf,
    pub store: CampaignStore,
    pub campaigns: CampaignList,
    /// The campaign currently open, if any.
    pub campaign: Option<Campaign>,
    pub status: Option<Status>,
}

impl AppContext {
    /// Loads the app config and opens the campaign store. Only a missing home directory is
    /// fatal. A damaged config falls back to the defaults and says so.
    pub fn load() -> Result<Self, StorageError> {
        let config_path = AppConfig::default_path()?;
        let (config, status) = match AppConfig::load(&config_path) {
            Ok(config) => (config, None),
            Err(e) => (
                AppConfig::default(),
                Some(Status::Error(format!("Using default settings: {e}"))),
            ),
        };
        let mut ctx = Self {
            store: CampaignStore::open(config.save_location)?,
            config,
            config_path,
            campaigns: CampaignList::default(),
            campaign: None,
            status,
        };
        ctx.refresh_campaigns();
        Ok(ctx)
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.status = Some(Status::Info(message.into()));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.status = Some(Status::Error(message.into()));
    }

    /// The open campaign's active Unbroken, if there is one.
    pub fn character(&self) -> Option<&Character> {
        self.campaign.as_ref()?.roster.active.as_ref()
    }

    pub fn character_mut(&mut self) -> Option<&mut Character> {
        self.campaign.as_mut()?.roster.active.as_mut()
    }

    /// Writes a journal entry and autosaves.
    pub fn record(&mut self, source: &str, text: impl Into<String>) {
        if let Some(campaign) = self.campaign.as_mut() {
            campaign.record(source, text);
            self.save_campaign();
        }
    }

    /// Re-reads the campaign folder.
    pub fn refresh_campaigns(&mut self) {
        match self.store.list() {
            Ok(list) => {
                if let Some(first) = list.unreadable.first() {
                    let others = list.unreadable.len() - 1;
                    let suffix = if others > 0 {
                        format!(" (+{others} more)")
                    } else {
                        String::new()
                    };
                    self.error(format!("Skipped a save: {first}{suffix}"));
                }
                self.campaigns = list;
            }
            Err(e) => {
                self.campaigns = CampaignList::default();
                self.error(format!("Couldn't read saves: {e}"));
            }
        }
    }

    /// Saves the open campaign. Called after every change (autosave).
    pub fn save_campaign(&mut self) {
        if let Some(campaign) = self.campaign.as_mut()
            && let Err(e) = self.store.save(campaign)
        {
            self.error(format!("Couldn't save: {e}"));
        }
    }

    /// Switches where campaigns are stored and remembers the choice.
    pub fn set_save_location(&mut self, location: SaveLocation) {
        let store = match CampaignStore::open(location) {
            Ok(store) => store,
            Err(e) => return self.error(format!("Couldn't use that location: {e}")),
        };
        self.config.save_location = location;
        if let Err(e) = self.config.save(&self.config_path) {
            self.error(format!("Couldn't save settings: {e}"));
        }
        self.store = store;
        self.refresh_campaigns();
    }
}
