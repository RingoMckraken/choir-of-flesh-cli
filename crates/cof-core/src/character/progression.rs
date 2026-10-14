//! Experience, levels and Development Points (p.30).

use super::attributes::BONUS_SKILL_SCORE;
use super::unbroken::SkillError;
use super::{Attribute, Character, Feat, LevelPurchases, Skill};
use crate::gear;

/// Development Points earned per level gained.
pub const DP_PER_LEVEL: u16 = 6;
pub const FEAT_COST: u16 = 4;
pub const PROFICIENCY_COST: u16 = 4;
pub const ATTRIBUTE_COST: u16 = 20;
pub const SKILL_COST: u16 = 5;
/// XP for ending a session (p.30), via [`Character::end_session`].
pub const SESSION_XP: u16 = 5;
/// XP for completing an Incursion (p.30).
pub const INCURSION_XP: u16 = 10;

/// XP needed to leave `level`: 50 at level 1, then 50 + level × 2 (the p.30 table: 54 to
/// leave level 2, 56 to leave level 3…). There's no level cap.
pub fn xp_to_next(level: u8) -> u16 {
    if level <= 1 {
        50
    } else {
        50 + 2 * u16::from(level)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PurchaseError {
    #[error("needs {need} DP, but only {have} left")]
    NotEnoughDp { need: u16, have: u16 },
    #[error("only one per level")]
    OncePerLevel,
    #[error("already known")]
    AlreadyHave,
    #[error("not a weapon or armor that needs a Proficiency")]
    NotProficiencyGear,
    #[error("no bonus Skill is owed for that Attribute")]
    NotOwed,
    #[error("that Skill isn't on the Attribute's list")]
    WrongList,
    #[error("{0}")]
    Skill(#[from] SkillError),
}

impl Character {
    /// XP still needed for the next level.
    pub fn xp_needed(&self) -> u16 {
        xp_to_next(self.level).saturating_sub(self.xp)
    }

    /// Adds XP and levels up as many times as it pays for, keeping the leftover. Each level
    /// brings 6 DP and resets the once-per-level purchases. Returns the levels reached.
    pub fn gain_xp(&mut self, amount: u16) -> Vec<u8> {
        self.xp = self.xp.saturating_add(amount);
        let mut reached = Vec::new();
        while self.xp >= xp_to_next(self.level) {
            self.xp -= xp_to_next(self.level);
            self.level = self.level.saturating_add(1);
            self.dp = self.dp.saturating_add(DP_PER_LEVEL);
            self.bought_this_level = LevelPurchases::default();
            reached.push(self.level);
        }
        reached
    }

    /// Takes XP away, to fix a mistake. Never undoes a level.
    pub fn lose_xp(&mut self, amount: u16) {
        self.xp = self.xp.saturating_sub(amount);
    }

    fn spend(&mut self, cost: u16) -> Result<(), PurchaseError> {
        if self.dp < cost {
            return Err(PurchaseError::NotEnoughDp {
                need: cost,
                have: self.dp,
            });
        }
        self.dp -= cost;
        Ok(())
    }

    /// A new Feat: 4 DP, once per level.
    pub fn buy_feat(&mut self, feat: Feat) -> Result<(), PurchaseError> {
        if self.bought_this_level.feat {
            return Err(PurchaseError::OncePerLevel);
        }
        if self.has_feat(feat) {
            return Err(PurchaseError::AlreadyHave);
        }
        self.spend(FEAT_COST)?;
        self.feats.push(feat);
        self.bought_this_level.feat = true;
        Ok(())
    }

    /// A weapon or armor Proficiency: 4 DP.
    pub fn buy_proficiency(&mut self, catalog_name: &str) -> Result<(), PurchaseError> {
        let needs_one = gear::weapons()
            .chain(gear::proficiency_armor())
            .any(|g| g.name == catalog_name);
        if !needs_one {
            return Err(PurchaseError::NotProficiencyGear);
        }
        if self.is_proficient(catalog_name) {
            return Err(PurchaseError::AlreadyHave);
        }
        self.spend(PROFICIENCY_COST)?;
        self.proficiencies.push(catalog_name.to_owned());
        Ok(())
    }

    /// +1 to an Attribute: 20 DP. Reaching 16 owes a bonus Skill from that Attribute's
    /// list (p.16), taken with [`Character::take_bonus_skill`].
    pub fn raise_attribute(&mut self, attribute: Attribute) -> Result<(), PurchaseError> {
        self.spend(ATTRIBUTE_COST)?;
        let score = self.attributes.score(attribute).saturating_add(1);
        self.attributes.set(attribute, score);
        if score == BONUS_SKILL_SCORE {
            self.pending_bonus_skills.push(attribute);
        }
        Ok(())
    }

    /// A new Skill: 5 DP, once per level.
    pub fn buy_skill(&mut self, skill: Skill) -> Result<(), PurchaseError> {
        if self.bought_this_level.skill {
            return Err(PurchaseError::OncePerLevel);
        }
        self.check_skill(skill)?;
        self.spend(SKILL_COST)?;
        self.learn_skill(skill)?;
        self.bought_this_level.skill = true;
        Ok(())
    }

    /// Takes a free bonus Skill owed for reaching 16 (p.16).
    pub fn take_bonus_skill(&mut self, skill: Skill) -> Result<(), PurchaseError> {
        let attribute = skill.attribute();
        let index = self
            .pending_bonus_skills
            .iter()
            .position(|&a| a == attribute)
            .ok_or(if self.pending_bonus_skills.is_empty() {
                PurchaseError::NotOwed
            } else {
                PurchaseError::WrongList
            })?;
        self.learn_skill(skill)?;
        self.pending_bonus_skills.remove(index);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::unbroken::fixture::character;

    #[test]
    fn the_level_table_on_page_30() {
        let table = [
            (1, 50),
            (2, 54),
            (3, 56),
            (4, 58),
            (5, 60),
            (9, 68),
            (10, 70),
        ];
        for (level, xp) in table {
            assert_eq!(xp_to_next(level), xp, "level {level}");
        }
        assert_eq!(xp_to_next(30), 110, "no level cap");
    }

    #[test]
    fn leftover_xp_carries_over() {
        let mut c = character();
        assert_eq!(c.gain_xp(49), Vec::<u8>::new());
        assert_eq!(c.gain_xp(1), vec![2]);
        assert_eq!((c.level, c.xp, c.dp), (2, 0, 6));
        // Patrick's example: at level 2, 60 XP pays 54 for level 3 and leaves 6.
        assert_eq!(c.gain_xp(60), vec![3]);
        assert_eq!((c.level, c.xp), (3, 6));
    }

    #[test]
    fn a_big_award_can_cross_several_levels() {
        let mut c = character();
        assert_eq!(c.gain_xp(50 + 54 + 56 + 3), vec![2, 3, 4]);
        assert_eq!((c.level, c.xp, c.dp), (4, 3, 18));
        assert_eq!(c.xp_needed(), 58 - 3);
    }

    #[test]
    fn feats_and_skills_once_per_level() {
        let mut c = character();
        c.gain_xp(50);
        c.dp = 20;
        assert!(c.buy_feat(Feat::Scout).is_ok());
        assert_eq!(c.buy_feat(Feat::Frugal), Err(PurchaseError::OncePerLevel));
        assert!(c.buy_skill(Skill::Tactics).is_ok());
        assert_eq!(c.buy_skill(Skill::Bluff), Err(PurchaseError::OncePerLevel));
        assert_eq!(c.dp, 20 - FEAT_COST - SKILL_COST);
        c.gain_xp(54);
        assert!(c.buy_feat(Feat::Frugal).is_ok());
        assert!(c.buy_skill(Skill::Bluff).is_ok());
    }

    #[test]
    fn purchases_need_the_dp_and_respect_the_rules() {
        let mut c = character();
        c.dp = 3;
        assert_eq!(
            c.buy_proficiency("Spear"),
            Err(PurchaseError::NotEnoughDp { need: 4, have: 3 })
        );
        c.dp = 8;
        assert_eq!(
            c.buy_proficiency("Spangenhelm"),
            Err(PurchaseError::NotProficiencyGear)
        );
        assert_eq!(
            c.buy_proficiency("Arming Sword"),
            Err(PurchaseError::AlreadyHave)
        );
        assert!(c.buy_proficiency("Spear").is_ok() && c.buy_proficiency("Light Shield").is_ok());
        c.dp = 5;
        c.drawbacks.push(crate::character::Drawback::Sickly);
        assert_eq!(
            c.buy_skill(Skill::Hardy),
            Err(PurchaseError::Skill(SkillError::SicklyBlocksHardy))
        );
        assert_eq!(c.dp, 5, "a refused purchase costs nothing");
    }

    #[test]
    fn reaching_16_owes_a_bonus_skill_from_that_list() {
        let mut c = character();
        c.attributes.set(Attribute::Dex, 15);
        c.dp = 40;
        c.raise_attribute(Attribute::Dex).unwrap();
        assert_eq!(c.pending_bonus_skills, [Attribute::Dex]);
        c.raise_attribute(Attribute::Dex).unwrap(); // 17: nothing more owed
        assert_eq!(c.pending_bonus_skills, [Attribute::Dex]);
        assert_eq!(
            c.take_bonus_skill(Skill::Tactics),
            Err(PurchaseError::WrongList)
        );
        assert!(c.take_bonus_skill(Skill::QuickDraw).is_ok());
        assert!(c.pending_bonus_skills.is_empty());
        assert_eq!(
            c.take_bonus_skill(Skill::Evasion),
            Err(PurchaseError::NotOwed)
        );
    }
}
