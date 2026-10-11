use serde::{Deserialize, Serialize};

use crate::rules::TnTable;

/// Rules options chosen per campaign.
///
/// App-wide options that apply before any campaign is open (like where saves live) are in
/// [`AppConfig`](crate::storage::AppConfig) instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CampaignSettings {
    /// Standard (p.43) or Scaling (p.230) Dynamic Target Numbers.
    #[serde(default)]
    pub tn_table: TnTable,
}
