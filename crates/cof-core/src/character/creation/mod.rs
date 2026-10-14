//! Character creation (p.12, p.224) as a draft the player fills in step by step. Core never
//! prompts: the TUI shows a step, passes the player's choices in, and asks the draft what's
//! still missing. [`Draft::build`] turns a finished draft into a [`Character`].

mod build;
mod choices;
mod shop;
mod skills;

pub use build::{Created, CreationError};
pub use choices::ChoiceError;
pub use shop::STARTING_RATIONS;
pub use skills::STARTING_SKILLS;

use serde::{Deserialize, Serialize};

use super::{AttributeRoll, Drawback, Feat, Relationship, Skill};

/// The steps, in the book's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    Attributes,
    Proficiencies,
    Feats,
    Skills,
    Background,
    Identity,
    Gear,
    Review,
}

impl Step {
    pub const ALL: [Step; 8] = [
        Step::Attributes,
        Step::Proficiencies,
        Step::Feats,
        Step::Skills,
        Step::Background,
        Step::Identity,
        Step::Gear,
        Step::Review,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Step::Attributes => "Attributes",
            Step::Proficiencies => "Proficiencies",
            Step::Feats => "Feats & Drawbacks",
            Step::Skills => "Skills",
            Step::Background => "Background",
            Step::Identity => "Name & Bonds",
            Step::Gear => "Starting Gear",
            Step::Review => "Review",
        }
    }
}

/// One Drawback taken: rolled (with its D12 result) or chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawbackPick {
    /// The D12 roll, or `None` when the player chose.
    pub roll: Option<u8>,
    /// `None` when a roll of 11–12 came up: no drawback, the character got lucky.
    pub drawback: Option<Drawback>,
}

/// A Gift of the Unmade taken in place of a Skill (p.12): always rolled, no Stigma, and it
/// costs D4 Humanity and 1 Anguish like any Blessing (Patrick's ruling, 2026-10-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreationGift {
    /// The D20 roll on the Gifts of the Unmade table.
    pub gift: u8,
    /// The D4 of Humanity it costs.
    pub humanity_loss: u8,
}

/// A character in the making.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// The six 4D6-drop-lowest rolls, assigned to Attributes in any order.
    pub rolls: [AttributeRoll; 6],
    /// Which roll each Attribute got, in sheet order (STR, CON, DEX, INT, WIL, PRE).
    pub assigned: [Option<usize>; 6],
    pub weapon_proficiency: Option<String>,
    pub armor_proficiency: Option<String>,
    /// The free Feat plus one per Drawback, at most three.
    pub feats: Vec<Feat>,
    pub drawbacks: Vec<DrawbackPick>,
    /// The four starting Skills (three after trading one for a Gift).
    pub skills: Vec<Skill>,
    pub gift: Option<CreationGift>,
    /// One per Attribute of 16+, from that Attribute's list.
    pub bonus_skills: Vec<Skill>,
    pub sin: Option<u8>,
    pub shard: Option<u8>,
    pub doom: Option<u8>,
    pub occupation: Option<u8>,
    pub occupation_skill: Option<Skill>,
    pub name: String,
    pub relationship: Option<Relationship>,
    /// The free weapon of the player's choice (p.12).
    pub starting_weapon: Option<String>,
    /// Gear bought with the 30 starting Rations: catalog name and quantity.
    pub cart: Vec<(String, u16)>,
}

impl Draft {
    /// A fresh draft with the Attribute dice already rolled.
    pub fn new() -> Self {
        Self {
            rolls: AttributeRoll::roll_six(),
            assigned: [None; 6],
            weapon_proficiency: None,
            armor_proficiency: None,
            feats: Vec::new(),
            drawbacks: Vec::new(),
            skills: Vec::new(),
            gift: None,
            bonus_skills: Vec::new(),
            sin: None,
            shard: None,
            doom: None,
            occupation: None,
            occupation_skill: None,
            name: String::new(),
            relationship: None,
            starting_weapon: None,
            cart: Vec::new(),
        }
    }
}

impl Default for Draft {
    fn default() -> Self {
        Self::new()
    }
}
