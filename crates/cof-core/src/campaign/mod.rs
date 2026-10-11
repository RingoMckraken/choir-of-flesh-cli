//! The Campaign: everything that is saved between sessions.
//!
//! In Choir of Flesh the **settlement is the protagonist**. Unbroken die often, and when one
//! falls a successor is raised from the settlement's population so the game can go on
//! indefinitely (the creator's intended way to play). So a campaign is named after its
//! settlement, and characters come and go within it. The character roster and settlement
//! are added to this struct in later milestones. New fields get serde defaults so older
//! saves keep loading.

mod settings;

pub use settings::CampaignSettings;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The save-file format this build writes. Bump it (and add a migration) only for changes
/// that serde defaults can't absorb.
pub const CURRENT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    pub format_version: u32,
    /// Stable identifier used for the save file name. Never changes, even on rename.
    pub id: String,
    /// The campaign's display name, usually its settlement's name.
    pub name: String,
    pub created: DateTime<Utc>,
    /// Last time the campaign was saved.
    pub updated: DateTime<Utc>,
    #[serde(default)]
    pub settings: CampaignSettings,
}

impl Campaign {
    /// A fresh campaign. Use [`CampaignStore::create`](crate::storage::CampaignStore::create)
    /// to get one with a unique id that's already saved.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            id: id.into(),
            name: name.into(),
            created: now,
            updated: now,
            settings: CampaignSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::TnTable;

    #[test]
    fn json_round_trip() {
        let mut campaign = Campaign::new("last-rest", "Last Rest");
        campaign.settings.tn_table = TnTable::Scaling;
        let json = serde_json::to_string(&campaign).unwrap();
        let back: Campaign = serde_json::from_str(&json).unwrap();
        assert_eq!(back, campaign);
    }

    #[test]
    fn missing_settings_fall_back_to_defaults() {
        // A save written before `settings` existed must still load.
        let json = r#"{
            "format_version": 1,
            "id": "old",
            "name": "Old Save",
            "created": "2026-10-11T12:00:00Z",
            "updated": "2026-10-11T12:00:00Z"
        }"#;
        let campaign: Campaign = serde_json::from_str(json).unwrap();
        assert_eq!(campaign.settings, CampaignSettings::default());
    }
}
