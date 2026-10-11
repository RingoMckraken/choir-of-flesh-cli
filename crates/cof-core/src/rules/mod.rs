//! Core rules (Chapter III): checks, Target Numbers, and the trackers that wear a character
//! down (Humanity, Anguish, Trauma), plus Conditions.

mod anguish;
mod breakdown;
mod checks;
mod conditions;
mod difficulty;
mod humanity;
mod target_number;
mod trauma;

pub use anguish::{Anguish, AnguishCheck, AnguishGain};
pub use breakdown::Breakdown;
pub use checks::{Check, CheckOutcome, CheckResult};
pub use conditions::Condition;
pub use difficulty::Difficulty;
pub use humanity::Humanity;
pub use target_number::TnTable;
pub use trauma::{Trauma, TraumaGain, TraumaTier};
