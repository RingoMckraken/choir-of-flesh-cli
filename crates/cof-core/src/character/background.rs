use serde::{Deserialize, Serialize};

use super::data::{OccupationEntry, TABLES, TitledEntry};
use crate::tables::RollTable;

/// The titled tables a character draws from. A character stores the *roll* (the entry's
/// number), and the text is looked up here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lore {
    /// A Sin That Defines You (D10, p.19).
    Sin,
    /// A Shard of the Old World (D10, p.21).
    Shard,
    /// A Whisper of Doom (D10, p.23).
    Doom,
    /// Existing Character Relationships (D10, p.29).
    Relationship,
    /// Gifts of the Unmade (D20, p.70–71).
    Gift,
    /// Stigmata of the Abyss (D20, p.72–73).
    Stigma,
}

impl Lore {
    pub fn name(self) -> &'static str {
        match self {
            Lore::Sin => "Sin",
            Lore::Shard => "Shard of the Old World",
            Lore::Doom => "Whisper of Doom",
            Lore::Relationship => "Relationship",
            Lore::Gift => "Gift of the Unmade",
            Lore::Stigma => "Stigma of the Abyss",
        }
    }

    pub fn table(self) -> &'static RollTable<TitledEntry> {
        let t = &*TABLES;
        match self {
            Lore::Sin => &t.sins,
            Lore::Shard => &t.shards,
            Lore::Doom => &t.dooms,
            Lore::Relationship => &t.relationships,
            Lore::Gift => &t.gifts,
            Lore::Stigma => &t.stigmata,
        }
    }

    /// The entry for a roll. Rolls come from the table itself or from a validated save, so
    /// an out-of-range roll is a bug.
    pub fn entry(self, roll: u8) -> &'static TitledEntry {
        self.table()
            .lookup(roll)
            .unwrap_or_else(|| panic!("{} has no entry {roll}", self.name()))
    }

    pub fn roll(self) -> u8 {
        self.table().roll().roll
    }

    /// How many entries the table has (10 or 20), for choosing instead of rolling.
    pub fn size(self) -> u8 {
        self.table().entries().count() as u8
    }
}

/// Gifts of the Unmade with an effect on derived stats (p.70–71).
pub mod gifts {
    /// +1 DR, permanently.
    pub const CHITINOUS_PLATES: u8 = 2;
    /// +5 inventory slots.
    pub const CORPULENT_STORAGE: u8 = 13;
    /// +1 Move.
    pub const SLITHERING_TAIL: u8 = 14;
}

/// The Former Occupation table (D10, p.24–25).
pub fn occupations() -> &'static RollTable<OccupationEntry> {
    &TABLES.occupations
}

pub fn occupation(roll: u8) -> &'static OccupationEntry {
    occupations()
        .lookup(roll)
        .unwrap_or_else(|| panic!("no occupation {roll}"))
}

/// A bond with another character (p.28–29). Optional; for a successor, the obvious "other"
/// is the Unbroken who came before.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub roll: u8,
    /// Who the bond is with, in the player's words.
    pub with: String,
}

/// Who the character was before the end (p.18–25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Background {
    pub sin: u8,
    pub shard: u8,
    pub doom: u8,
    pub occupation: u8,
    #[serde(default)]
    pub relationship: Option<Relationship>,
}

impl Background {
    pub fn sin(&self) -> &'static TitledEntry {
        Lore::Sin.entry(self.sin)
    }

    pub fn shard(&self) -> &'static TitledEntry {
        Lore::Shard.entry(self.shard)
    }

    pub fn doom(&self) -> &'static TitledEntry {
        Lore::Doom.entry(self.doom)
    }

    pub fn occupation(&self) -> &'static OccupationEntry {
        occupation(self.occupation)
    }

    pub fn relationship(&self) -> Option<&'static TitledEntry> {
        self.relationship
            .as_ref()
            .map(|r| Lore::Relationship.entry(r.roll))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const ALL: [Lore; 6] = [
        Lore::Sin,
        Lore::Shard,
        Lore::Doom,
        Lore::Relationship,
        Lore::Gift,
        Lore::Stigma,
    ];

    #[test]
    fn rolls_reach_every_entry() {
        for lore in ALL {
            let mut seen = HashSet::new();
            for _ in 0..2_000 {
                let roll = lore.roll();
                assert!(!lore.entry(roll).title.is_empty());
                seen.insert(roll);
            }
            assert_eq!(seen, (1..=lore.size()).collect(), "{}", lore.name());
        }
    }

    #[test]
    fn gift_constants_point_at_the_right_entries() {
        assert_eq!(
            Lore::Gift.entry(gifts::CHITINOUS_PLATES).title,
            "Chitinous Plates"
        );
        assert_eq!(
            Lore::Gift.entry(gifts::CORPULENT_STORAGE).title,
            "Corpulent Storage"
        );
        assert_eq!(
            Lore::Gift.entry(gifts::SLITHERING_TAIL).title,
            "Slithering Tail"
        );
    }

    #[test]
    fn a_background_looks_up_its_text() {
        let b = Background {
            sin: 2,
            shard: 1,
            doom: 9,
            occupation: 4,
            relationship: Some(Relationship {
                roll: 1,
                with: "Godwin".into(),
            }),
        };
        assert_eq!(b.sin().title, "Wrath");
        assert_eq!(b.shard().title, "A Rusted Crucifix");
        assert_eq!(b.doom().title, "The Black Bell");
        assert_eq!(b.occupation().title, "Soldier");
        assert_eq!(b.relationship().unwrap().title, "Kin");
    }
}
