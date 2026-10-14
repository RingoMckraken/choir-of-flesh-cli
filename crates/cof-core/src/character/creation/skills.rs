use super::choices::ChoiceError;
use super::{CreationGift, Draft};
use crate::character::{Attribute, Lore, Skill, SkillError, occupation, skill_allowed};
use crate::dice::Die;

/// Starting Skill picks before any trade for a Gift (p.12).
pub const STARTING_SKILLS: usize = 4;

impl Draft {
    /// Every Skill picked so far: the general picks, the bonus ones and the occupation's.
    pub fn all_skills(&self) -> Vec<Skill> {
        self.skills
            .iter()
            .chain(&self.bonus_skills)
            .chain(&self.occupation_skill)
            .copied()
            .collect()
    }

    /// Whether `skill` can still be added anywhere (no duplicates, no Hardy with Sickly).
    pub fn skill_available(&self, skill: Skill) -> Result<(), SkillError> {
        skill_allowed(&self.all_skills(), &self.taken_drawbacks(), skill)
    }

    /// How many general picks the draft gets: 4, or 3 after trading one for a Gift.
    pub fn general_skill_slots(&self) -> usize {
        STARTING_SKILLS - usize::from(self.gift.is_some())
    }

    /// Adds a general Skill pick, or removes it if already picked.
    pub fn toggle_skill(&mut self, skill: Skill) -> Result<(), ChoiceError> {
        if let Some(i) = self.skills.iter().position(|&s| s == skill) {
            self.skills.remove(i);
            return Ok(());
        }
        if self.skills.len() >= self.general_skill_slots() {
            return Err(ChoiceError::Full);
        }
        self.skill_available(skill).map_err(skill_error)?;
        self.skills.push(skill);
        Ok(())
    }

    /// Trades one Skill pick for a Gift of the Unmade (p.12): rolls the D20 Gift and the
    /// D4 of Humanity it costs. Rolled, so it can't be taken back. If all four picks are
    /// made, the last one is dropped to make room.
    pub fn trade_skill_for_gift(&mut self) -> Result<CreationGift, ChoiceError> {
        if self.gift.is_some() {
            return Err(ChoiceError::Duplicate);
        }
        let gift = CreationGift {
            gift: Lore::Gift.roll(),
            humanity_loss: Die::D4.roll(),
        };
        self.gift = Some(gift);
        self.skills.truncate(self.general_skill_slots());
        Ok(gift)
    }

    /// The Attributes of 16+ that each owe a bonus Skill from their own list.
    pub fn bonus_skill_attributes(&self) -> Vec<Attribute> {
        self.attributes()
            .map(|a| a.bonus_skill_attributes())
            .unwrap_or_default()
    }

    /// The bonus Skill picked for `attribute`, if any.
    pub fn bonus_skill(&self, attribute: Attribute) -> Option<Skill> {
        self.bonus_skills
            .iter()
            .copied()
            .find(|s| s.attribute() == attribute)
    }

    /// Picks (or replaces) the bonus Skill for the Attribute it belongs to.
    pub fn set_bonus_skill(&mut self, skill: Skill) -> Result<(), ChoiceError> {
        let attribute = skill.attribute();
        if !self.bonus_skill_attributes().contains(&attribute) {
            return Err(ChoiceError::NotAllowed);
        }
        let previous = self.bonus_skill(attribute);
        self.bonus_skills.retain(|s| s.attribute() != attribute);
        if let Err(e) = self.skill_available(skill) {
            self.bonus_skills.extend(previous);
            return Err(skill_error(e));
        }
        self.bonus_skills.push(skill);
        Ok(())
    }

    /// Drops bonus Skills whose Attribute is no longer 16+ (after reassigning rolls).
    pub fn prune_bonus_skills(&mut self) {
        let owed = self.bonus_skill_attributes();
        self.bonus_skills.retain(|s| owed.contains(&s.attribute()));
    }

    /// The occupation's Skills the draft could still take.
    pub fn occupation_skill_options(&self) -> Vec<Skill> {
        let Some(roll) = self.occupation else {
            return Vec::new();
        };
        let mut others = self.clone();
        others.occupation_skill = None;
        occupation(roll)
            .skills
            .iter()
            .copied()
            .filter(|&s| others.skill_available(s).is_ok())
            .collect()
    }

    /// Picks the occupation's Skill (p.24).
    pub fn set_occupation_skill(&mut self, skill: Skill) -> Result<(), ChoiceError> {
        if !self.occupation_skill_options().contains(&skill) {
            return Err(ChoiceError::NotAllowed);
        }
        self.occupation_skill = Some(skill);
        Ok(())
    }
}

fn skill_error(e: SkillError) -> ChoiceError {
    match e {
        SkillError::AlreadyKnown => ChoiceError::Duplicate,
        SkillError::SicklyBlocksHardy => ChoiceError::SicklyHardy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{AttributeRoll, Drawback};
    use std::collections::HashSet;

    /// A draft whose Attributes are fixed: STR 16, DEX 17, the rest 10.
    fn draft() -> Draft {
        let mut d = Draft::new();
        let roll = |a, b, c| AttributeRoll { dice: [1, a, b, c] };
        d.rolls = [
            roll(6, 5, 5),
            roll(4, 3, 3),
            roll(6, 6, 5),
            roll(4, 3, 3),
            roll(4, 3, 3),
            roll(4, 3, 3),
        ];
        for (i, a) in Attribute::ALL.into_iter().enumerate() {
            d.assign(a, i);
        }
        d
    }

    #[test]
    fn four_general_picks_without_duplicates() {
        let mut d = draft();
        for s in [Skill::Athlete, Skill::Stealth, Skill::Calm, Skill::Bluff] {
            d.toggle_skill(s).unwrap();
        }
        assert_eq!(d.toggle_skill(Skill::Awe), Err(ChoiceError::Full));
        d.toggle_skill(Skill::Bluff).unwrap();
        d.bonus_skills.push(Skill::Awe);
        assert_eq!(d.toggle_skill(Skill::Awe), Err(ChoiceError::Duplicate));
    }

    #[test]
    fn trading_for_a_gift_rolls_it_and_drops_a_pick() {
        let mut gifts = HashSet::new();
        let mut losses = HashSet::new();
        for _ in 0..2_000 {
            let mut d = draft();
            for s in [Skill::Athlete, Skill::Stealth, Skill::Calm, Skill::Bluff] {
                d.toggle_skill(s).unwrap();
            }
            let gift = d.trade_skill_for_gift().unwrap();
            assert!((1..=20).contains(&gift.gift) && (1..=4).contains(&gift.humanity_loss));
            gifts.insert(gift.gift);
            losses.insert(gift.humanity_loss);
            assert_eq!(d.skills.len(), 3);
            assert_eq!(d.trade_skill_for_gift(), Err(ChoiceError::Duplicate));
        }
        assert_eq!((gifts.len(), losses.len()), (20, 4));
    }

    #[test]
    fn bonus_skills_come_from_the_high_attributes_lists() {
        let mut d = draft();
        assert_eq!(d.bonus_skill_attributes(), [Attribute::Str, Attribute::Dex]);
        assert_eq!(d.set_bonus_skill(Skill::Calm), Err(ChoiceError::NotAllowed));
        d.set_bonus_skill(Skill::Athlete).unwrap();
        d.set_bonus_skill(Skill::Brace).unwrap(); // replaces Athlete
        d.set_bonus_skill(Skill::Stealth).unwrap();
        assert_eq!(d.bonus_skills, [Skill::Brace, Skill::Stealth]);
        d.assign(Attribute::Dex, 1); // DEX drops to 10, CON gets the 17
        d.prune_bonus_skills();
        assert_eq!(d.bonus_skills, [Skill::Brace]);
    }

    #[test]
    fn occupation_options_skip_known_skills_and_sickly_hardy() {
        let mut d = draft();
        d.set_occupation(1); // Peasant: Heavy Lifting, Hardy, Insight
        d.toggle_skill(Skill::Insight).unwrap();
        d.choose_drawback(Drawback::Sickly).unwrap();
        assert_eq!(d.occupation_skill_options(), [Skill::HeavyLifting]);
        assert_eq!(
            d.set_occupation_skill(Skill::Hardy),
            Err(ChoiceError::NotAllowed)
        );
        d.set_occupation_skill(Skill::HeavyLifting).unwrap();
        assert_eq!(
            d.occupation_skill_options(),
            [Skill::HeavyLifting],
            "the current pick stays an option"
        );
    }

    #[test]
    fn hardy_is_refused_with_sickly() {
        let mut d = draft();
        d.choose_drawback(Drawback::Sickly).unwrap();
        assert_eq!(d.toggle_skill(Skill::Hardy), Err(ChoiceError::SicklyHardy));
    }
}
