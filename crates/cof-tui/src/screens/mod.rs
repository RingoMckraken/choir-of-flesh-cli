//! The app's screens. Tools get their own `tools/` folders as they're built.

mod app_settings;
mod campaign_hub;
mod campaign_settings;
mod coming_soon;
mod main_menu;
mod new_campaign;

pub use main_menu::MainMenu;

use chrono::{DateTime, Local, Utc};

/// A timestamp in the player's local time, e.g. "Oct 11 2026, 14:05".
fn local_time(time: DateTime<Utc>) -> String {
    time.with_timezone(&Local)
        .format("%b %-d %Y, %H:%M")
        .to_string()
}
