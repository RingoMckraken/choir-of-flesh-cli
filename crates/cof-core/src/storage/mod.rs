//! Saving and loading: campaign files and the app's own config.
//!
//! Campaigns are pretty-printed JSON, one file per campaign, written atomically (temp file +
//! rename) with the previous version kept as `<id>.json.bak`.

mod app_config;
mod atomic;
mod campaign_store;
mod error;
mod location;

pub use app_config::AppConfig;
pub use campaign_store::{CampaignList, CampaignStore, CampaignSummary};
pub use error::StorageError;
pub use location::SaveLocation;
