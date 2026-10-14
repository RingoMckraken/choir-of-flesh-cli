//! The Campaign: everything that is saved between sessions.
//!
//! In Choir of Flesh the **settlement is the protagonist**. Unbroken die often, and when one
//! falls a successor is raised from the settlement's population so the game can go on
//! indefinitely (the creator's intended way to play). So a campaign is named after its
//! settlement, and characters come and go within it (the [`Roster`]). The settlement is
//! added in a later milestone. New fields get serde defaults so older
//! saves keep loading.

mod journal;
mod settings;
mod trackers;

pub use journal::JournalEntry;
pub use settings::CampaignSettings;
pub use trackers::UsageTracker;

use chrono::{DateTime, Utc};

use crate::character::Roster;
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
    /// Everything rolled and decided, oldest first.
    #[serde(default)]
    pub journal: Vec<JournalEntry>,
    /// Named Usage Dice the player is tracking.
    #[serde(default)]
    pub usage_dice: Vec<UsageTracker>,
    /// The active Unbroken and the memorial of the fallen.
    #[serde(default)]
    pub roster: Roster,
    /// Rumors gathered (p.102). They belong to the campaign, so they outlive a death; 10 of
    /// them lead to an Incursion (p.110).
    #[serde(default)]
    pub rumors: u16,
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
            journal: Vec::new(),
            usage_dice: Vec::new(),
            roster: Roster::default(),
            rumors: 0,
        }
    }

    /// Adds an entry to the journal.
    pub fn record(&mut self, source: impl Into<String>, text: impl Into<String>) {
        self.journal.push(JournalEntry {
            at: Utc::now(),
            source: source.into(),
            text: text.into(),
        });
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
        campaign.record("Yes/No Oracle", "Likely (60%): D100 = 12 → YES");
        campaign.usage_dice.push(UsageTracker::new(
            "Torch",
            crate::dice::UsageDie::new(crate::dice::Die::D4).unwrap(),
        ));
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
        assert!(campaign.journal.is_empty() && campaign.usage_dice.is_empty());
        assert!(campaign.roster.active.is_none() && campaign.roster.fallen.is_empty());
        assert_eq!(campaign.rumors, 0);
    }
}
