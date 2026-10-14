//! The sheet's day-to-day actions: Anguish, Humanity, Trauma, Vigor and session upkeep.

use super::Character;
use super::progression::SESSION_XP;
use crate::rules::{AnguishGain, TraumaGain};

impl Character {
    /// Gains Anguish the way the rules say: Cowardly and Broken Trauma add 1 each, the
    /// Anguish Check gets +2 from Calm and -1 while Shaken (p.15, p.18, p.47–48, p.61).
    /// A Breakdown makes a Catharsis attempt available (p.48).
    pub fn gain_anguish(&mut self, amount: u8) -> AnguishGain {
        let total = if amount == 0 {
            0
        } else {
            amount.saturating_add(self.extra_anguish())
        };
        let gain = self.anguish.gain(total, self.anguish_check_modifier());
        self.note_breakdown(&gain);
        gain
    }

    /// The Voice Within (p.49): a permanent point of Anguish, with its own Anguish Check.
    pub fn gain_permanent_anguish(&mut self) -> AnguishGain {
        let gain = self.anguish.gain_permanent(self.anguish_check_modifier());
        self.note_breakdown(&gain);
        gain
    }

    fn note_breakdown(&mut self, gain: &AnguishGain) {
        if gain.check.is_some_and(|c| c.breakdown.is_some()) {
            self.catharsis_available = true;
        }
    }

    /// An Act of Defiance: +1 Humanity, once per session (p.46). False if already used.
    pub fn act_of_defiance(&mut self) -> bool {
        if self.session.defiance_used {
            return false;
        }
        self.session.defiance_used = true;
        self.humanity.regain(1);
        true
    }

    /// The Lucky feat's reroll, once per session (p.15). False if already used.
    pub fn use_lucky(&mut self) -> bool {
        if self.session.lucky_used {
            return false;
        }
        self.session.lucky_used = true;
        true
    }

    pub fn gain_trauma(&mut self, amount: u8) -> TraumaGain {
        self.trauma.gain(amount)
    }

    /// Spends a point of Vigor (p.56). False if none is left.
    pub fn spend_vigor(&mut self) -> bool {
        if self.vigor() == 0 {
            return false;
        }
        self.vigor_spent += 1;
        true
    }

    /// Six hours of rest: all Vigor comes back (p.56).
    pub fn rest(&mut self) {
        self.vigor_spent = 0;
    }

    /// Ends a play session: +5 XP (p.30) and the once-per-session flags reset. Returns any
    /// levels reached.
    pub fn end_session(&mut self) -> Vec<u8> {
        self.session = Default::default();
        self.gain_xp(SESSION_XP)
    }

    /// Humanity 0: the character is lost to the Choir or the Flesh (p.47).
    pub fn is_lost(&self) -> bool {
        self.humanity.is_lost()
    }

    /// Trauma 13 or more: the body gives out (p.61).
    pub fn is_dead_of_trauma(&self) -> bool {
        self.trauma.is_fatal()
    }
}

#[cfg(test)]
mod tests {
    use crate::character::unbroken::fixture::character;
    use crate::character::{Drawback, Skill};

    #[test]
    fn anguish_gain_applies_extras_and_check_modifier() {
        for _ in 0..3_000 {
            let mut c = character(); // Calm: +2 to the check
            c.drawbacks.push(Drawback::Cowardly);
            let gain = c.gain_anguish(1);
            assert_eq!(gain.gained, 2, "Cowardly adds one");
            let check = gain.check.unwrap();
            assert_eq!(check.total, i16::from(check.natural) + 2);
            assert_eq!(check.breakdown.is_some(), check.total <= 2);
            assert_eq!(c.catharsis_available, check.breakdown.is_some());
        }
    }

    #[test]
    fn broken_trauma_adds_anguish_and_shaken_lowers_the_check() {
        let mut c = character();
        c.skills.retain(|&s| s != Skill::Calm);
        c.gain_trauma(9);
        let gain = c.gain_anguish(2);
        assert_eq!(gain.gained, 3);
        let check = gain.check.unwrap();
        assert_eq!(check.total, i16::from(check.natural) - 1);
    }

    #[test]
    fn nothing_gained_means_no_check() {
        let mut c = character();
        c.drawbacks.push(Drawback::Cowardly);
        let gain = c.gain_anguish(0);
        assert_eq!((gain.gained, gain.check), (0, None));
    }

    #[test]
    fn once_per_session_until_the_session_ends() {
        let mut c = character();
        c.humanity.lose(2);
        assert!(c.act_of_defiance() && !c.act_of_defiance());
        assert!(c.use_lucky() && !c.use_lucky());
        assert_eq!(c.humanity.current(), 19);
        assert!(c.end_session().is_empty());
        assert_eq!(c.xp, 5);
        assert!(c.act_of_defiance() && c.use_lucky());
    }

    #[test]
    fn vigor_spends_down_and_rest_refills() {
        let mut c = character();
        c.attributes.set(crate::character::Attribute::Con, 16); // +2, WIL +0
        assert!(c.spend_vigor() && c.spend_vigor() && !c.spend_vigor());
        c.rest();
        assert_eq!(c.vigor(), 2);
    }

    #[test]
    fn fatal_states() {
        let mut c = character();
        assert!(!c.is_lost() && !c.is_dead_of_trauma());
        c.humanity.lose(20);
        c.gain_trauma(13);
        assert!(c.is_lost() && c.is_dead_of_trauma());
    }
}
