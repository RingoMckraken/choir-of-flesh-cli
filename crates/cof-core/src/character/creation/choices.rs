use super::{Draft, DrawbackPick};
use crate::character::{Attribute, Attributes, Drawback, Feat, Skill, occupation};

/// Why a choice was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChoiceError {
    #[error("already taken")]
    Duplicate,
    #[error("no more room")]
    Full,
    #[error("the Sickly drawback and the Hardy skill can't go together")]
    SicklyHardy,
    #[error("that isn't allowed here")]
    NotAllowed,
}

impl Draft {
    /// Gives `attribute` the roll at `roll_index`. If another Attribute had that roll, the
    /// two swap.
    pub fn assign(&mut self, attribute: Attribute, roll_index: usize) {
        let slot = attribute as usize;
        if let Some(other) = self.assigned.iter().position(|&r| r == Some(roll_index)) {
            self.assigned[other] = self.assigned[slot];
        }
        self.assigned[slot] = Some(roll_index);
    }

    /// The scores, once every Attribute has a roll.
    pub fn attributes(&self) -> Option<Attributes> {
        let mut scores = [0; 6];
        for (score, assigned) in scores.iter_mut().zip(self.assigned) {
            *score = self.rolls[assigned?].total();
        }
        Some(Attributes::new(scores))
    }

    /// The Drawbacks actually taken (lucky 11–12 rolls excluded).
    pub fn taken_drawbacks(&self) -> Vec<Drawback> {
        self.drawbacks.iter().filter_map(|p| p.drawback).collect()
    }

    /// How many Feats the draft gets: 1, +1 per Drawback, at most 3 (p.12).
    pub fn feat_allowance(&self) -> usize {
        (1 + self.taken_drawbacks().len()).min(Feat::CREATION_MAX)
    }

    /// Adds the Feat, or removes it if already picked.
    pub fn toggle_feat(&mut self, feat: Feat) -> Result<(), ChoiceError> {
        if let Some(i) = self.feats.iter().position(|&f| f == feat) {
            self.feats.remove(i);
            return Ok(());
        }
        if self.feats.len() >= self.feat_allowance() {
            return Err(ChoiceError::Full);
        }
        self.feats.push(feat);
        Ok(())
    }

    fn has_hardy(&self) -> bool {
        self.all_skills().contains(&Skill::Hardy)
    }

    fn drawback_refused(&self, drawback: Drawback) -> Option<ChoiceError> {
        if self.taken_drawbacks().contains(&drawback) {
            Some(ChoiceError::Duplicate)
        } else if drawback == Drawback::Sickly && self.has_hardy() {
            Some(ChoiceError::SicklyHardy)
        } else {
            None
        }
    }

    /// Rolls on the Drawback table (D12, p.15). Drawbacks the draft can't take (already
    /// taken, or Sickly alongside Hardy) are rerolled. 11–12 means no drawback at all.
    pub fn roll_drawback(&mut self) -> Result<DrawbackPick, ChoiceError> {
        if self.drawbacks.len() >= Drawback::CREATION_MAX {
            return Err(ChoiceError::Full);
        }
        let pick = loop {
            let (roll, drawback) = Drawback::roll();
            if drawback.is_none_or(|d| self.drawback_refused(d).is_none()) {
                break DrawbackPick {
                    roll: Some(roll),
                    drawback,
                };
            }
        };
        self.drawbacks.push(pick);
        Ok(pick)
    }

    /// Picks a Drawback outright.
    pub fn choose_drawback(&mut self, drawback: Drawback) -> Result<(), ChoiceError> {
        if self.drawbacks.len() >= Drawback::CREATION_MAX {
            return Err(ChoiceError::Full);
        }
        if let Some(e) = self.drawback_refused(drawback) {
            return Err(e);
        }
        self.drawbacks.push(DrawbackPick {
            roll: None,
            drawback: Some(drawback),
        });
        Ok(())
    }

    /// Takes back a Drawback, dropping the last Feats if the allowance shrinks.
    pub fn remove_drawback(&mut self, index: usize) {
        if index < self.drawbacks.len() {
            self.drawbacks.remove(index);
            self.feats.truncate(self.feat_allowance());
        }
    }

    /// Sets the Former Occupation, clearing the occupation Skill if it isn't on the new list.
    pub fn set_occupation(&mut self, roll: u8) {
        self.occupation = Some(roll);
        if let Some(skill) = self.occupation_skill
            && !occupation(roll).skills.contains(&skill)
        {
            self.occupation_skill = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn assigning_a_used_roll_swaps() {
        let mut d = Draft::new();
        for (i, a) in Attribute::ALL.into_iter().enumerate() {
            d.assign(a, i);
        }
        let before = d.attributes().unwrap();
        d.assign(Attribute::Str, 5);
        let after = d.attributes().unwrap();
        assert_eq!(after.score(Attribute::Str), before.score(Attribute::Pre));
        assert_eq!(after.score(Attribute::Pre), before.score(Attribute::Str));
        d.assigned[3] = None;
        assert!(d.attributes().is_none());
    }

    #[test]
    fn feats_follow_drawbacks_up_to_three() {
        let mut d = Draft::new();
        assert_eq!(d.feat_allowance(), 1);
        d.toggle_feat(Feat::Lucky).unwrap();
        assert_eq!(d.toggle_feat(Feat::Scout), Err(ChoiceError::Full));
        for drawback in [Drawback::Mute, Drawback::Slow, Drawback::Clumsy] {
            d.choose_drawback(drawback).unwrap();
        }
        assert_eq!(
            d.feat_allowance(),
            3,
            "the third Drawback earns nothing more"
        );
        assert_eq!(
            d.choose_drawback(Drawback::Gullible),
            Err(ChoiceError::Full)
        );
        d.toggle_feat(Feat::Scout).unwrap();
        d.toggle_feat(Feat::Frugal).unwrap();
        d.remove_drawback(0);
        d.remove_drawback(0);
        assert_eq!(d.feats, [Feat::Lucky, Feat::Scout]);
        d.toggle_feat(Feat::Lucky).unwrap();
        assert_eq!(d.feats, [Feat::Scout]);
    }

    #[test]
    fn rolled_drawbacks_never_repeat_and_can_be_lucky() {
        let mut faces = HashSet::new();
        for _ in 0..3_000 {
            let mut d = Draft::new();
            for _ in 0..3 {
                let pick = d.roll_drawback().unwrap();
                let roll = pick.roll.unwrap();
                faces.insert(roll);
                assert_eq!(pick.drawback, Drawback::from_roll(roll));
            }
            let taken = d.taken_drawbacks();
            let unique: HashSet<_> = taken.iter().collect();
            assert_eq!(unique.len(), taken.len());
            assert!(d.roll_drawback().is_err());
        }
        assert_eq!(faces, (1..=12).collect());
    }

    #[test]
    fn sickly_never_joins_hardy() {
        let mut d = Draft::new();
        d.skills.push(Skill::Hardy);
        assert_eq!(
            d.choose_drawback(Drawback::Sickly),
            Err(ChoiceError::SicklyHardy)
        );
        for _ in 0..500 {
            let mut d2 = d.clone();
            d2.roll_drawback().unwrap();
            assert!(!d2.taken_drawbacks().contains(&Drawback::Sickly));
        }
    }

    #[test]
    fn changing_occupation_drops_a_skill_not_on_its_list() {
        let mut d = Draft::new();
        d.set_occupation(4); // Soldier
        d.occupation_skill = Some(Skill::BattleFrenzy);
        d.set_occupation(4);
        assert_eq!(d.occupation_skill, Some(Skill::BattleFrenzy));
        d.set_occupation(8); // Scholar
        assert_eq!(d.occupation_skill, None);
    }
}
