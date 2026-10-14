use serde::{Deserialize, Serialize};

use super::Breakdown;
use crate::dice::Die;

/// Anguish (p.47–48): the slow erosion of a character's soul. Starts at 0.
///
/// Every time Anguish is gained, the character makes an **Anguish Check**: roll a D20, and
/// if the result is less than or equal to the new total, they suffer a [`Breakdown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Anguish {
    current: u8,
    /// Points that can never be removed (from The Voice Within, p.49).
    permanent: u8,
    /// Set by defying a Moment of Clarity: the next gain is doubled (p.49).
    double_next_gain: bool,
}

/// The result of gaining Anguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnguishGain {
    /// How much was actually added (after any doubling).
    pub gained: u8,
    pub total: u8,
    /// `None` only when nothing was gained.
    pub check: Option<AnguishCheck>,
}

/// One Anguish Check (p.48).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnguishCheck {
    pub natural: u8,
    /// Natural roll plus any modifiers (e.g. +2 from Calm, -1 from Shaken Trauma).
    pub total: i16,
    /// `Some` when the check failed and the character broke.
    pub breakdown: Option<Breakdown>,
}

impl Anguish {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    /// Anguish that can't be removed by any means.
    pub fn permanent(&self) -> u8 {
        self.permanent
    }

    /// True after defying a Moment of Clarity, until the next gain.
    pub fn next_gain_doubled(&self) -> bool {
        self.double_next_gain
    }

    /// Gains Anguish and immediately makes the Anguish Check.
    ///
    /// `check_modifier` adjusts the D20 roll: +2 for the Calm skill (p.18), -1 Burden while
    /// Shaken by Trauma (p.61). Extra Anguish from the Cowardly drawback or Broken Trauma
    /// is the caller's job to add to `amount`.
    pub fn gain(&mut self, amount: u8, check_modifier: i8) -> AnguishGain {
        if amount == 0 {
            return AnguishGain {
                gained: 0,
                total: self.current,
                check: None,
            };
        }
        let gained = if self.double_next_gain {
            self.double_next_gain = false;
            amount.saturating_mul(2)
        } else {
            amount
        };
        self.current = self.current.saturating_add(gained);
        AnguishGain {
            gained,
            total: self.current,
            check: Some(self.check(check_modifier)),
        }
    }

    /// The Voice Within (p.49): gain a permanent point of Anguish. It counts as gaining
    /// Anguish, so it triggers an Anguish Check too.
    pub fn gain_permanent(&mut self, check_modifier: i8) -> AnguishGain {
        self.permanent = self.permanent.saturating_add(1);
        self.gain(1, check_modifier)
    }

    /// Loses Anguish, but never below the permanent points.
    pub fn lose(&mut self, amount: u8) {
        self.current = self.current.saturating_sub(amount).max(self.permanent);
    }

    /// Defying a Moment of Clarity (p.49): lose all Anguish (except permanent points), but
    /// the next Anguish gained is doubled.
    pub fn defy_clarity(&mut self) {
        self.current = self.permanent;
        self.double_next_gain = true;
    }

    fn check(&self, modifier: i8) -> AnguishCheck {
        let natural = Die::D20.roll();
        let total = i16::from(natural) + i16::from(modifier);
        let breakdown = (total <= i16::from(self.current)).then(Breakdown::roll);
        AnguishCheck {
            natural,
            total,
            breakdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROLLS: usize = 20_000;

    fn with(current: u8) -> Anguish {
        Anguish {
            current,
            ..Anguish::default()
        }
    }

    /// Gains `amount` on a fresh tracker at `start` many times, asserting the Anguish Check
    /// rule on every roll. Returns how many checks broke.
    fn run_checks(start: u8, amount: u8, modifier: i8) -> usize {
        let mut breaks = 0;
        let mut faces = [false; 20];
        for _ in 0..ROLLS {
            let mut a = with(start);
            let gain = a.gain(amount, modifier);
            let check = gain.check.expect("gaining Anguish always makes a check");
            faces[usize::from(check.natural) - 1] = true;
            assert_eq!(check.total, i16::from(check.natural) + i16::from(modifier));
            let should_break = check.total <= i16::from(gain.total);
            assert_eq!(check.breakdown.is_some(), should_break, "{check:?}");
            breaks += usize::from(should_break);
        }
        assert!(faces.iter().all(|&f| f), "some D20 face never came up");
        breaks
    }

    #[test]
    fn rulebook_example_breaks_on_five_or_less() {
        // 4 Anguish, gains 1 → 5: rolls of 1–5 break, 6+ hold (p.48). About 25% break.
        let breaks = run_checks(4, 1, 0);
        assert!(breaks > 0 && breaks < ROLLS / 2);
    }

    #[test]
    fn calm_and_shaken_modifiers_apply_to_every_roll() {
        run_checks(4, 1, 2);
        run_checks(4, 1, -1);
    }

    #[test]
    fn twenty_anguish_always_breaks() {
        assert_eq!(run_checks(19, 1, 0), ROLLS);
    }

    #[test]
    fn gaining_nothing_makes_no_check() {
        let mut a = with(3);
        let gain = a.gain(0, 0);
        assert_eq!((gain.gained, gain.total, gain.check), (0, 3, None));
    }

    #[test]
    fn lose_saturates_at_zero() {
        let mut a = with(2);
        a.lose(5);
        assert_eq!(a.current(), 0);
    }

    #[test]
    fn permanent_anguish_cannot_be_lost() {
        let mut a = Anguish::new();
        a.gain_permanent(0);
        a.gain(3, 0);
        assert_eq!((a.current(), a.permanent()), (4, 1));
        a.lose(10);
        assert_eq!(a.current(), 1);
        a.defy_clarity();
        assert_eq!(a.current(), 1);
    }

    #[test]
    fn defying_clarity_clears_then_doubles_next_gain() {
        let mut a = with(9);
        a.defy_clarity();
        assert_eq!(a.current(), 0);
        assert!(a.next_gain_doubled());

        let gain = a.gain(2, 0);
        assert_eq!((gain.gained, gain.total), (4, 4));
        assert!(!a.next_gain_doubled());

        assert_eq!(a.gain(2, 0).gained, 2);
    }
}
