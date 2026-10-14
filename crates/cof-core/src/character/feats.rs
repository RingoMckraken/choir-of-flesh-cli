use serde::{Deserialize, Serialize};

use super::data::DESCRIPTIONS;

/// The Feats (p.14–15). Descriptions are in `data/character/feats.ron`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Feat {
    Backstab,
    Brawler,
    DualWielder,
    Explorer,
    Firm,
    Frugal,
    Healer,
    KeenSenses,
    Lucky,
    Marksman,
    NaturalBornLeader,
    Nimble,
    Pathfinder,
    PrecisionWarrior,
    Provider,
    QuickHands,
    Resilient,
    Scout,
    Skirmisher,
    Stealthy,
}

impl Feat {
    /// Starting characters have at most this many Feats, counting the free one (p.12).
    /// Feats bought later with Development Points aren't capped.
    pub const CREATION_MAX: usize = 3;

    pub const ALL: [Feat; 20] = [
        Feat::Backstab,
        Feat::Brawler,
        Feat::DualWielder,
        Feat::Explorer,
        Feat::Firm,
        Feat::Frugal,
        Feat::Healer,
        Feat::KeenSenses,
        Feat::Lucky,
        Feat::Marksman,
        Feat::NaturalBornLeader,
        Feat::Nimble,
        Feat::Pathfinder,
        Feat::PrecisionWarrior,
        Feat::Provider,
        Feat::QuickHands,
        Feat::Resilient,
        Feat::Scout,
        Feat::Skirmisher,
        Feat::Stealthy,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Feat::Backstab => "Backstab",
            Feat::Brawler => "Brawler",
            Feat::DualWielder => "Dual Wielder",
            Feat::Explorer => "Explorer",
            Feat::Firm => "Firm",
            Feat::Frugal => "Frugal",
            Feat::Healer => "Healer",
            Feat::KeenSenses => "Keen Senses",
            Feat::Lucky => "Lucky",
            Feat::Marksman => "Marksman",
            Feat::NaturalBornLeader => "Natural Born Leader",
            Feat::Nimble => "Nimble",
            Feat::Pathfinder => "Pathfinder",
            Feat::PrecisionWarrior => "Precision Warrior",
            Feat::Provider => "Provider",
            Feat::QuickHands => "Quick Hands",
            Feat::Resilient => "Resilient",
            Feat::Scout => "Scout",
            Feat::Skirmisher => "Skirmisher",
            Feat::Stealthy => "Stealthy",
        }
    }

    pub fn description(self) -> &'static str {
        &DESCRIPTIONS.feats[&self]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_feat_has_a_description() {
        for feat in Feat::ALL {
            assert!(!feat.description().is_empty(), "{feat:?}");
        }
        assert_eq!(DESCRIPTIONS.feats.len(), Feat::ALL.len());
        assert_eq!(Feat::Skirmisher.description(), "Increase your Move by +1.");
    }
}
