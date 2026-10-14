use serde::{Deserialize, Serialize};

use crate::rules::TnTable;

/// Rules options chosen per campaign.
///
/// App-wide options that apply before any campaign is open (like where saves live) are in
/// [`AppConfig`](crate::storage::AppConfig) instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignSettings {
    /// Standard (p.43) or Scaling (p.230) Dynamic Target Numbers.
    #[serde(default)]
    pub tn_table: TnTable,
    /// Patrick's house rule: when an Unbroken dies or retires, their successor may take one
    /// item they carried. On by default; strict play turns it off.
    #[serde(default = "on")]
    pub heirloom: bool,
}

impl Default for CampaignSettings {
    fn default() -> Self {
        Self {
            tn_table: TnTable::default(),
            heirloom: true,
        }
    }
}

fn on() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heirloom_is_on_for_saves_that_predate_it() {
        let old: CampaignSettings = serde_json::from_str(r#"{"tn_table":"Scaling"}"#).unwrap();
        assert!(old.heirloom);
        assert_eq!(old.tn_table, TnTable::Scaling);
    }
}
