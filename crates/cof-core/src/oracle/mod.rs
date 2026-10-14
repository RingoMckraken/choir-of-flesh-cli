//! Solo Narrative Tools (p.84–93): the GM-less oracle, prompts, and NPC generator, plus the
//! Undertaking generator (p.75–83).
//!
//! Every procedure returns a result that implements `Display`, which is a ready-made journal
//! entry.

mod data;
mod npc;
mod prompts;
mod undertaking;
mod yes_no;

pub use data::{NamePair, SurpriseRow, UndertakingEntry, name_table};
pub use npc::{Disposition, Npc, generate_npc};
pub use prompts::{
    ActionResolution, ActionTheme, Interest, SurpriseEvent, action_theme, anything_of_interest,
    resolve_action, surprise_event,
};
pub use undertaking::{
    Undertaking, UndertakingRoll, UndertakingStep, undertaking, undertaking_step,
};
pub use yes_no::{Likelihood, OracleAnswer, Twist, ask};
