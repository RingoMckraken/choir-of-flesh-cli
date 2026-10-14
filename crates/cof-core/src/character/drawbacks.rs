use serde::{Deserialize, Serialize};

use super::data::DESCRIPTIONS;
use crate::dice::Die;

/// The Drawbacks (p.15). Each one taken at creation earns an extra Feat. Descriptions are in
/// `data/character/drawbacks.ron`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Drawback {
    Arrogant,
    Clumsy,
    Cowardly,
    Gullible,
    Ignorant,
    MissingEye,
    Mute,
    Sickly,
    SlowHealer,
    Slow,
}

impl Drawback {
    /// A new character may take at most this many Drawbacks (p.224).
    pub const CREATION_MAX: usize = 3;

    /// Table order: index 0 is a roll of 1. 11–12 is "No Drawback".
    pub const ALL: [Drawback; 10] = [
        Drawback::Arrogant,
        Drawback::Clumsy,
        Drawback::Cowardly,
        Drawback::Gullible,
        Drawback::Ignorant,
        Drawback::MissingEye,
        Drawback::Mute,
        Drawback::Sickly,
        Drawback::SlowHealer,
        Drawback::Slow,
    ];

    /// Looks up a D12 result (p.15). 11–12 is "No Drawback": the character got lucky.
    pub fn from_roll(roll: u8) -> Option<Drawback> {
        Self::ALL.get(usize::from(roll).checked_sub(1)?).copied()
    }

    /// Rolls the D12 Drawback table. Returns the roll too, so it can be shown.
    pub fn roll() -> (u8, Option<Drawback>) {
        let roll = Die::D12.roll();
        (roll, Self::from_roll(roll))
    }

    pub fn name(self) -> &'static str {
        match self {
            Drawback::Arrogant => "Arrogant",
            Drawback::Clumsy => "Clumsy",
            Drawback::Cowardly => "Cowardly",
            Drawback::Gullible => "Gullible",
            Drawback::Ignorant => "Ignorant",
            Drawback::MissingEye => "Missing Eye",
            Drawback::Mute => "Mute",
            Drawback::Sickly => "Sickly",
            Drawback::SlowHealer => "Slow Healer",
            Drawback::Slow => "Slow",
        }
    }

    pub fn description(self) -> &'static str {
        &DESCRIPTIONS.drawbacks[&self]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_face_of_the_d12() {
        for roll in 1..=12 {
            let drawback = Drawback::from_roll(roll);
            assert_eq!(drawback.is_none(), roll >= 11, "roll {roll}");
        }
        assert_eq!(Drawback::from_roll(1), Some(Drawback::Arrogant));
        assert_eq!(Drawback::from_roll(10), Some(Drawback::Slow));
        assert_eq!(Drawback::from_roll(0), None);
    }

    #[test]
    fn real_rolls_reach_every_result() {
        let mut seen = HashSet::new();
        for _ in 0..5_000 {
            let (roll, drawback) = Drawback::roll();
            assert_eq!(drawback, Drawback::from_roll(roll));
            seen.insert(roll);
        }
        assert_eq!(seen, (1..=12).collect());
    }

    #[test]
    fn every_drawback_has_a_description() {
        for d in Drawback::ALL {
            assert!(!d.description().is_empty(), "{d:?}");
        }
        assert!(Drawback::Sickly.description().ends_with("Hardy Skill."));
    }
}
