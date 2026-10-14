use std::collections::HashSet;

use super::{CreationGift, Draft, Step};
use crate::character::{
    Background, Character, Drawback, Inventory, Item, LevelPurchases, SessionFlags, skill_allowed,
};
use crate::gear::{self, GAMBESON};
use crate::rules::{Anguish, AnguishGain, Humanity, Trauma};

/// A step isn't finished yet.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}: {problem}", step.name())]
pub struct CreationError {
    pub step: Step,
    pub problem: String,
}

/// A freshly made Unbroken, plus what the Gift cost them, for the wizard to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Created {
    pub character: Character,
    pub gift: Option<(CreationGift, AnguishGain)>,
}

impl Draft {
    /// What's still missing in a step, in words for the player. `None` means it's done.
    pub fn problem(&self, step: Step) -> Option<String> {
        match step {
            Step::Attributes => self
                .attributes()
                .is_none()
                .then(|| "assign a roll to every Attribute".into()),
            Step::Proficiencies => match (&self.weapon_proficiency, &self.armor_proficiency) {
                (None, _) => Some("pick a weapon Proficiency".into()),
                (_, None) => Some("pick an armor Proficiency".into()),
                _ => None,
            },
            Step::Feats => {
                let allowance = self.feat_allowance();
                (self.feats.len() != allowance).then(|| {
                    format!(
                        "pick {allowance} Feat{}",
                        if allowance == 1 { "" } else { "s" }
                    )
                })
            }
            Step::Skills => self.skills_problem(),
            Step::Background => self.background_problem(),
            Step::Identity => self
                .name
                .trim()
                .is_empty()
                .then(|| "name your Unbroken".into()),
            Step::Gear => self
                .starting_weapon
                .is_none()
                .then(|| "pick your free weapon".into()),
            Step::Review => Step::ALL[..Step::ALL.len() - 1]
                .iter()
                .find_map(|&s| self.problem(s).map(|p| format!("{}: {p}", s.name()))),
        }
    }

    fn skills_problem(&self) -> Option<String> {
        let slots = self.general_skill_slots();
        if self.skills.len() != slots {
            return Some(format!("pick {slots} Skills"));
        }
        let owed = self.bonus_skill_attributes();
        if let Some(a) = owed.iter().find(|&&a| self.bonus_skill(a).is_none()) {
            return Some(format!(
                "pick a bonus {} Skill for your score of 16+",
                a.abbr()
            ));
        }
        None
    }

    fn background_problem(&self) -> Option<String> {
        if self.sin.is_none() || self.shard.is_none() || self.doom.is_none() {
            return Some("roll or choose your Sin, Shard and Whisper of Doom".into());
        }
        if self.occupation.is_none() {
            return Some("roll or choose your Former Occupation".into());
        }
        if self.occupation_skill.is_none() && !self.occupation_skill_options().is_empty() {
            return Some("pick your occupation's Skill".into());
        }
        None
    }

    /// Turns a finished draft into a character. Applies the Gift's price: D4 Humanity and
    /// 1 Anguish, which makes an Anguish Check (and could, rarely, break them already).
    pub fn build(self) -> Result<Created, CreationError> {
        for step in Step::ALL {
            if let Some(problem) = self.problem(step) {
                return Err(CreationError { step, problem });
            }
        }
        let invalid = |step, problem: &str| CreationError {
            step,
            problem: problem.into(),
        };
        let drawbacks: Vec<Drawback> = self.taken_drawbacks();
        let skills = self.all_skills();
        let mut seen = Vec::new();
        for &skill in &skills {
            skill_allowed(&seen, &drawbacks, skill)
                .map_err(|e| invalid(Step::Skills, &e.to_string()))?;
            seen.push(skill);
        }
        if self.feats.iter().collect::<HashSet<_>>().len() != self.feats.len() {
            return Err(invalid(Step::Feats, "a Feat is picked twice"));
        }

        let mut inventory = Inventory {
            items: Vec::new(),
            rations: self.rations_left(),
        };
        let weapon = self.starting_weapon.as_deref().unwrap_or_default();
        for name in [weapon, GAMBESON] {
            let gear = gear::find(name).ok_or_else(|| invalid(Step::Gear, "unknown gear"))?;
            let mut item = Item::from_catalog(gear, 1);
            item.equipped = true;
            inventory.add(item);
        }
        for (name, qty) in &self.cart {
            let gear = gear::find(name).ok_or_else(|| invalid(Step::Gear, "unknown gear"))?;
            inventory.add(Item::from_catalog(gear, *qty));
        }

        let attributes = self.attributes().expect("checked above");
        let mut character = Character {
            name: self.name.trim().to_owned(),
            attributes,
            level: 1,
            xp: 0,
            dp: 0,
            proficiencies: [&self.weapon_proficiency, &self.armor_proficiency]
                .into_iter()
                .flatten()
                .cloned()
                .collect(),
            feats: self.feats,
            drawbacks,
            skills,
            gifts: self.gift.map(|g| g.gift).into_iter().collect(),
            stigmata: Vec::new(),
            background: Background {
                sin: self.sin.expect("checked"),
                shard: self.shard.expect("checked"),
                doom: self.doom.expect("checked"),
                occupation: self.occupation.expect("checked"),
                relationship: self.relationship,
            },
            humanity: Humanity::new(),
            anguish: Anguish::new(),
            trauma: Trauma::new(),
            vigor_spent: 0,
            inventory,
            phobias: Vec::new(),
            catharsis_available: false,
            session: SessionFlags::default(),
            bought_this_level: LevelPurchases::default(),
            pending_bonus_skills: Vec::new(),
            notes: String::new(),
        };
        character.humanity = Humanity::new_with_max(character.max_humanity());
        let gift = self.gift.map(|gift| {
            character.humanity.lose(gift.humanity_loss);
            (gift, character.gain_anguish(1))
        });
        Ok(Created { character, gift })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Attribute, AttributeRoll, Feat, Lore, Skill};
    use std::collections::HashSet;

    /// A complete draft: every step filled in, nothing at 16+.
    fn finished() -> Draft {
        let mut d = Draft::new();
        d.rolls = [AttributeRoll { dice: [1, 4, 4, 4] }; 6];
        for (i, a) in Attribute::ALL.into_iter().enumerate() {
            d.assign(a, i);
        }
        d.weapon_proficiency = Some("Spear".into());
        d.armor_proficiency = Some("Gambeson".into());
        d.toggle_feat(Feat::Lucky).unwrap();
        for s in [Skill::Athlete, Skill::Stealth, Skill::Healing, Skill::Bluff] {
            d.toggle_skill(s).unwrap();
        }
        (d.sin, d.shard, d.doom) = (Some(1), Some(2), Some(3));
        d.set_occupation(4);
        d.set_occupation_skill(Skill::BattleFrenzy).unwrap();
        d.name = "  Hilda  ".into();
        d.set_starting_weapon("Spear").unwrap();
        d.buy("Torch").unwrap();
        d
    }

    #[test]
    fn every_step_reports_what_is_missing() {
        let d = Draft::new();
        for step in Step::ALL {
            if step != Step::Feats || d.feat_allowance() > 0 {
                assert!(d.problem(step).is_some(), "{step:?}");
            }
        }
        let d = finished();
        for step in Step::ALL {
            assert_eq!(d.problem(step), None, "{step:?}");
        }
    }

    #[test]
    fn a_finished_draft_builds_a_level_one_character() {
        let created = finished().build().unwrap();
        let c = created.character;
        assert!(created.gift.is_none());
        assert_eq!((c.name.as_str(), c.level, c.xp, c.dp), ("Hilda", 1, 0, 0));
        assert_eq!(c.proficiencies, ["Spear", "Gambeson"]);
        assert_eq!(c.skills.len(), 5);
        assert_eq!(c.inventory.rations, 28);
        assert_eq!(c.inventory.count("Torch"), 1);
        let equipped: Vec<_> = c
            .inventory
            .equipped_gear()
            .map(|g| g.name.as_str())
            .collect();
        assert_eq!(equipped, ["Spear", "Gambeson"]);
        assert_eq!(c.defense_rating(), 1, "DEX +0, Gambeson +1");
        assert_eq!((c.humanity.current(), c.anguish.current()), (20, 0));
    }

    #[test]
    fn empathic_starts_at_22() {
        let mut d = finished();
        d.toggle_skill(Skill::Bluff).unwrap();
        d.toggle_skill(Skill::Empathic).unwrap();
        let c = d.build().unwrap().character;
        assert_eq!((c.humanity.current(), c.humanity.max()), (22, 22));
    }

    #[test]
    fn the_gift_costs_humanity_and_anguish() {
        let mut breakdowns = HashSet::new();
        for _ in 0..3_000 {
            let mut d = finished();
            let gift = d.trade_skill_for_gift().unwrap();
            let created = d.build().unwrap();
            let c = &created.character;
            let (g, anguish) = created.gift.unwrap();
            assert_eq!(g, gift);
            assert_eq!(c.gifts, [gift.gift]);
            assert!(!Lore::Gift.entry(gift.gift).title.is_empty());
            assert!(c.stigmata.is_empty(), "no Stigma for a creation Gift");
            assert_eq!(c.humanity.current(), 20 - gift.humanity_loss);
            assert_eq!((anguish.gained, c.anguish.current()), (1, 1));
            let broke = anguish.check.unwrap().breakdown.is_some();
            assert_eq!(c.catharsis_available, broke);
            breakdowns.insert(broke);
        }
        assert_eq!(
            breakdowns.len(),
            2,
            "a natural 1 breaks them even at creation"
        );
    }

    #[test]
    fn cowardly_makes_the_gift_cost_two_anguish() {
        let mut d = finished();
        d.choose_drawback(Drawback::Cowardly).unwrap();
        d.toggle_feat(Feat::Scout).unwrap();
        d.trade_skill_for_gift().unwrap();
        let created = d.build().unwrap();
        assert_eq!(created.character.anguish.current(), 2);
    }

    #[test]
    fn an_unfinished_draft_names_the_step() {
        let mut d = finished();
        d.name.clear();
        assert_eq!(d.build().unwrap_err().step, Step::Identity);
    }
}
