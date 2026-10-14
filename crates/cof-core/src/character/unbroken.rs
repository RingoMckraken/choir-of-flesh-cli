use serde::{Deserialize, Serialize};

use super::{Attribute, Attributes, Background, Drawback, Feat, Inventory, Skill};
use crate::rules::{Anguish, Humanity, Trauma};

/// Base maximum Humanity, before the Empathic skill (p.18, p.46).
pub const BASE_MAX_HUMANITY: u8 = 20;

/// Once-per-session flags, cleared by [`Character::end_session`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SessionFlags {
    /// The Lucky feat's reroll (p.15).
    #[serde(default)]
    pub lucky_used: bool,
    /// An Act of Defiance's +1 Humanity (p.46).
    #[serde(default)]
    pub defiance_used: bool,
}

/// Development Point purchases limited to one per level (p.30).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LevelPurchases {
    #[serde(default)]
    pub feat: bool,
    #[serde(default)]
    pub skill: bool,
}

/// An Unbroken: one of the settlement's line of player characters.
///
/// Derived stats (modifiers, DR, Initiative, Vigor, slots…) are methods, never stored, so
/// they can't go stale. Read the fields freely, but change Skills through
/// [`Character::learn_skill`] so the rules that hang off them (Empathic, Sickly/Hardy) hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub attributes: Attributes,
    pub level: u8,
    /// XP toward the next level. Leftover XP carries over on a level-up.
    pub xp: u16,
    /// Unspent Development Points.
    pub dp: u16,
    /// Catalog names of the weapons and armor this character is proficient with.
    pub proficiencies: Vec<String>,
    pub feats: Vec<Feat>,
    pub drawbacks: Vec<Drawback>,
    pub skills: Vec<Skill>,
    /// Gifts of the Unmade, by table number (p.70–71).
    #[serde(default)]
    pub gifts: Vec<u8>,
    /// Stigmata of the Abyss, by table number (p.72–73).
    #[serde(default)]
    pub stigmata: Vec<u8>,
    pub background: Background,
    pub humanity: Humanity,
    pub anguish: Anguish,
    pub trauma: Trauma,
    /// Vigor spent since the last 6-hour rest (p.56).
    #[serde(default)]
    pub vigor_spent: u8,
    #[serde(default)]
    pub inventory: Inventory,
    /// Permanent phobias from the Breakdown table (p.49), removed by Defying Your Doom.
    #[serde(default)]
    pub phobias: Vec<String>,
    /// Set after a Breakdown: the next safe moment allows one Catharsis attempt (p.48).
    #[serde(default)]
    pub catharsis_available: bool,
    #[serde(default)]
    pub session: SessionFlags,
    #[serde(default)]
    pub bought_this_level: LevelPurchases,
    /// Bonus Skills owed for reaching 16 in an Attribute after creation (p.16).
    #[serde(default)]
    pub pending_bonus_skills: Vec<Attribute>,
    /// Free-form notes: injuries, timed effects, the story so far.
    #[serde(default)]
    pub notes: String,
}

/// Why a Skill can't be learned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SkillError {
    #[error("already known")]
    AlreadyKnown,
    #[error("the Sickly drawback rules out Hardy")]
    SicklyBlocksHardy,
}

impl Character {
    pub fn has_skill(&self, skill: Skill) -> bool {
        self.skills.contains(&skill)
    }

    pub fn has_feat(&self, feat: Feat) -> bool {
        self.feats.contains(&feat)
    }

    pub fn has_drawback(&self, drawback: Drawback) -> bool {
        self.drawbacks.contains(&drawback)
    }

    pub fn is_proficient(&self, catalog_name: &str) -> bool {
        self.proficiencies.iter().any(|p| p == catalog_name)
    }

    /// Whether `skill` could be learned now.
    pub fn check_skill(&self, skill: Skill) -> Result<(), SkillError> {
        skill_allowed(&self.skills, &self.drawbacks, skill)
    }

    /// Learns a Skill, applying its standing effects (Empathic raises max Humanity).
    pub fn learn_skill(&mut self, skill: Skill) -> Result<(), SkillError> {
        self.check_skill(skill)?;
        self.skills.push(skill);
        self.humanity.set_max(self.max_humanity());
        Ok(())
    }

    /// Max Humanity: 20, or 22 with Empathic (p.18).
    pub fn max_humanity(&self) -> u8 {
        BASE_MAX_HUMANITY
            + if self.has_skill(Skill::Empathic) {
                2
            } else {
                0
            }
    }
}

/// The Skill rules shared by creation and play: no duplicates (except Suppress The Change)
/// and no Hardy with Sickly (Patrick's ruling on the "Hardy Feat" erratum, p.15).
pub fn skill_allowed(
    known: &[Skill],
    drawbacks: &[Drawback],
    skill: Skill,
) -> Result<(), SkillError> {
    if known.contains(&skill) && !skill.repeatable() {
        return Err(SkillError::AlreadyKnown);
    }
    if skill == Skill::Hardy && drawbacks.contains(&Drawback::Sickly) {
        return Err(SkillError::SicklyBlocksHardy);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod fixture {
    use super::*;

    /// A plain level 1 character for tests.
    pub fn character() -> Character {
        Character {
            name: "Godwin".into(),
            attributes: Attributes::new([12, 10, 14, 9, 11, 8]),
            level: 1,
            xp: 0,
            dp: 0,
            proficiencies: vec!["Arming Sword".into(), "Gambeson".into()],
            feats: vec![Feat::Lucky],
            drawbacks: vec![],
            skills: vec![Skill::Athlete, Skill::Stealth, Skill::Healing, Skill::Calm],
            gifts: vec![],
            stigmata: vec![],
            background: Background {
                sin: 2,
                shard: 1,
                doom: 9,
                occupation: 4,
                relationship: None,
            },
            humanity: Humanity::new(),
            anguish: Anguish::new(),
            trauma: Trauma::new(),
            vigor_spent: 0,
            inventory: Inventory::default(),
            phobias: vec![],
            catharsis_available: false,
            session: SessionFlags::default(),
            bought_this_level: LevelPurchases::default(),
            pending_bonus_skills: vec![],
            notes: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixture::character;
    use super::*;

    #[test]
    fn skills_are_unique_except_suppress_the_change() {
        let mut c = character();
        assert_eq!(c.learn_skill(Skill::Athlete), Err(SkillError::AlreadyKnown));
        assert!(c.learn_skill(Skill::SuppressTheChange).is_ok());
        assert!(c.learn_skill(Skill::SuppressTheChange).is_ok());
    }

    #[test]
    fn sickly_blocks_hardy() {
        let mut c = character();
        c.drawbacks.push(Drawback::Sickly);
        assert_eq!(
            c.learn_skill(Skill::Hardy),
            Err(SkillError::SicklyBlocksHardy)
        );
        c.drawbacks.clear();
        assert!(c.learn_skill(Skill::Hardy).is_ok());
    }

    #[test]
    fn empathic_raises_the_humanity_cap_only() {
        let mut c = character();
        c.humanity.lose(3);
        c.learn_skill(Skill::Empathic).unwrap();
        assert_eq!((c.humanity.current(), c.humanity.max()), (17, 22));
    }

    #[test]
    fn survives_a_save() {
        let c = character();
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(serde_json::from_str::<Character>(&json).unwrap(), c);
    }
}
