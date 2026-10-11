//! Solo Narrative Tools (p.84–93): the GM-less oracle, prompts, and NPC generator.
//!
//! Every procedure returns a result that implements `Display`, which is a ready-made journal
//! entry.

mod data;
mod npc;
mod prompts;
mod yes_no;

pub use data::{NamePair, SurpriseRow};
pub use npc::{Disposition, Npc, generate_npc};
pub use prompts::{
    ActionResolution, ActionTheme, Interest, SurpriseEvent, action_theme, anything_of_interest,
    resolve_action, surprise_event,
};
pub use yes_no::{Likelihood, OracleAnswer, Twist, ask};
