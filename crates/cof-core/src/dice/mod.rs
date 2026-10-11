//! Dice: the physical dice of the game and the Usage Die.

mod die;
mod usage_die;

pub use die::{Die, RollMode, Rolled};
pub use usage_die::{UsageDie, UsageOutcome, UsageRoll};
