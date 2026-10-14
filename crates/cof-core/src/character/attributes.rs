use serde::{Deserialize, Serialize};

use crate::dice::Die;

/// The six Attributes (p.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Attribute {
    Str,
    Con,
    Dex,
    Int,
    Wil,
    Pre,
}

impl Attribute {
    /// Character sheet order.
    pub const ALL: [Attribute; 6] = [
        Attribute::Str,
        Attribute::Con,
        Attribute::Dex,
        Attribute::Int,
        Attribute::Wil,
        Attribute::Pre,
    ];

    /// The short form used everywhere in the book, e.g. "STR".
    pub fn abbr(self) -> &'static str {
        match self {
            Attribute::Str => "STR",
            Attribute::Con => "CON",
            Attribute::Dex => "DEX",
            Attribute::Int => "INT",
            Attribute::Wil => "WIL",
            Attribute::Pre => "PRE",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Attribute::Str => "Strength",
            Attribute::Con => "Constitution",
            Attribute::Dex => "Dexterity",
            Attribute::Int => "Intelligence",
            Attribute::Wil => "Willpower",
            Attribute::Pre => "Presence",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// The Attribute modifier table (p.13).
pub fn modifier(score: u8) -> i8 {
    match score {
        0..=3 => -3,
        4..=5 => -2,
        6..=8 => -1,
        9..=12 => 0,
        13..=15 => 1,
        16..=17 => 2,
        _ => 3,
    }
}

/// The score at which an Attribute grants a bonus Skill from its own list (p.16, p.224).
pub const BONUS_SKILL_SCORE: u8 = 16;

/// A character's six Attribute scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Attributes {
    scores: [u8; 6],
}

impl Attributes {
    /// Scores in sheet order (STR, CON, DEX, INT, WIL, PRE).
    pub fn new(scores: [u8; 6]) -> Self {
        Self { scores }
    }

    pub fn score(&self, attribute: Attribute) -> u8 {
        self.scores[attribute.index()]
    }

    pub fn modifier(&self, attribute: Attribute) -> i8 {
        modifier(self.score(attribute))
    }

    pub fn set(&mut self, attribute: Attribute, score: u8) {
        self.scores[attribute.index()] = score;
    }

    /// Attributes at 16 or more, each worth a bonus Skill from its own list.
    pub fn bonus_skill_attributes(&self) -> Vec<Attribute> {
        Attribute::ALL
            .into_iter()
            .filter(|&a| self.score(a) >= BONUS_SKILL_SCORE)
            .collect()
    }
}

/// One Attribute roll at creation: 4D6, drop the lowest, add the other three (p.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRoll {
    pub dice: [u8; 4],
}

impl AttributeRoll {
    pub fn roll() -> Self {
        Self {
            dice: [
                Die::D6.roll(),
                Die::D6.roll(),
                Die::D6.roll(),
                Die::D6.roll(),
            ],
        }
    }

    /// The die that's dropped (the lowest).
    pub fn dropped(&self) -> u8 {
        *self.dice.iter().min().expect("four dice")
    }

    pub fn total(&self) -> u8 {
        self.dice.iter().sum::<u8>() - self.dropped()
    }

    /// The six rolls for a new character, to be assigned to Attributes in any order.
    pub fn roll_six() -> [AttributeRoll; 6] {
        std::array::from_fn(|_| AttributeRoll::roll())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn modifier_table_matches_page_13() {
        let expected = [
            (3, -3),
            (4, -2),
            (5, -2),
            (6, -1),
            (8, -1),
            (9, 0),
            (12, 0),
            (13, 1),
            (15, 1),
            (16, 2),
            (17, 2),
            (18, 3),
            (19, 3),
        ];
        for (score, m) in expected {
            assert_eq!(modifier(score), m, "score {score}");
        }
    }

    #[test]
    fn four_d6_drop_lowest_covers_3_to_18() {
        let mut totals = HashSet::new();
        for _ in 0..50_000 {
            let r = AttributeRoll::roll();
            assert!(r.dice.iter().all(|d| (1..=6).contains(d)));
            let mut sorted = r.dice;
            sorted.sort_unstable();
            assert_eq!(r.total(), sorted[1] + sorted[2] + sorted[3]);
            totals.insert(r.total());
        }
        assert_eq!(totals, (3..=18).collect());
    }

    #[test]
    fn scores_are_set_and_read_by_attribute() {
        let mut a = Attributes::new([10, 11, 12, 13, 14, 16]);
        assert_eq!(a.score(Attribute::Int), 13);
        assert_eq!(a.modifier(Attribute::Pre), 2);
        a.set(Attribute::Str, 17);
        assert_eq!(
            a.bonus_skill_attributes(),
            vec![Attribute::Str, Attribute::Pre]
        );
    }
}
