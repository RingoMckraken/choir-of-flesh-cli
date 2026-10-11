use super::{Difficulty, TnTable};
use crate::dice::{Die, RollMode};

/// A check (p.42): roll a D20, add modifiers, and compare with a Target Number.
///
/// Build one with [`Check::static_tn`] or [`Check::dynamic`], add Mastery, Burden, or
/// Advantage, then [`roll`](Check::roll) it, or [`resolve`](Check::resolve) a D20 the player
/// rolled themselves.
///
/// ```
/// use cof_core::rules::{Check, CheckResult, Difficulty, TnTable};
///
/// // Isabella picks an Average lock with DEX +1 and a +2 Mastery toolkit (p.45).
/// let check = Check::dynamic(TnTable::Standard, Difficulty::Average, 1).with_mastery(2);
/// assert_eq!(check.tn, 12);
/// assert_eq!(check.resolve(9).result, CheckResult::Success); // "only needs a 9 or higher"
/// assert_eq!(check.resolve(8).result, CheckResult::Failure);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Check {
    pub tn: u8,
    /// The character's total modifier (usually an Attribute modifier).
    pub modifier: i8,
    /// "+X Mastery" (p.45), added to the roll after the TN is set.
    pub mastery: u8,
    /// "-X Burden" (p.45), subtracted from the roll after the TN is set. Stored as a
    /// positive amount: "-2 Burden" is `burden: 2`.
    pub burden: u8,
    pub mode: RollMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    CriticalSuccess,
    Success,
    Failure,
    CriticalFailure,
}

impl CheckResult {
    pub fn is_success(self) -> bool {
        matches!(self, CheckResult::CriticalSuccess | CheckResult::Success)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOutcome {
    pub result: CheckResult,
    /// The D20 result that counted.
    pub natural: u8,
    /// The other D20, when rolling with Advantage or Disadvantage.
    pub discarded: Option<u8>,
    /// Natural roll + modifier + Mastery - Burden.
    pub total: i16,
    pub tn: u8,
    /// `total - tn`. Zero or more is a success unless the natural roll was a 1. Combat uses
    /// the size of a miss to decide Minor Injuries (p.55).
    pub margin: i16,
}

impl Check {
    /// A check against a fixed Target Number.
    pub fn static_tn(tn: u8, modifier: i8) -> Self {
        Self {
            tn,
            modifier,
            mastery: 0,
            burden: 0,
            mode: RollMode::Normal,
        }
    }

    /// A check whose TN comes from a Dynamic Target Number table (p.42, p.230).
    pub fn dynamic(table: TnTable, difficulty: Difficulty, modifier: i8) -> Self {
        Self::static_tn(table.target_number(difficulty, modifier), modifier)
    }

    pub fn with_mastery(mut self, mastery: u8) -> Self {
        self.mastery = self.mastery.saturating_add(mastery);
        self
    }

    pub fn with_burden(mut self, burden: u8) -> Self {
        self.burden = self.burden.saturating_add(burden);
        self
    }

    pub fn with_mode(mut self, mode: RollMode) -> Self {
        self.mode = mode;
        self
    }

    /// Rolls the D20 (twice for Advantage or Disadvantage) and resolves the check.
    pub fn roll(&self) -> CheckOutcome {
        let rolled = Die::D20.roll_with(self.mode);
        CheckOutcome {
            discarded: rolled.discarded,
            ..self.resolve(rolled.kept)
        }
    }

    /// Resolves the check for a natural D20 result that has already been rolled.
    ///
    /// A natural 20 always succeeds and a natural 1 always fails (p.44).
    pub fn resolve(&self, natural: u8) -> CheckOutcome {
        let total = i16::from(natural) + i16::from(self.modifier) + i16::from(self.mastery)
            - i16::from(self.burden);
        let margin = total - i16::from(self.tn);
        let result = match natural {
            20 => CheckResult::CriticalSuccess,
            1 => CheckResult::CriticalFailure,
            _ if margin >= 0 => CheckResult::Success,
            _ => CheckResult::Failure,
        };
        CheckOutcome {
            result,
            natural,
            discarded: None,
            total,
            tn: self.tn,
            margin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROLLS: usize = 20_000;

    /// Asserts that `check` succeeds on exactly the natural rolls from `lowest_success` up
    /// (plus the automatic natural 20), checking every face of the D20.
    fn assert_succeeds_from(check: Check, lowest_success: u8) {
        for natural in 1..=20 {
            let outcome = check.resolve(natural);
            let expected = match natural {
                20 => CheckResult::CriticalSuccess,
                1 => CheckResult::CriticalFailure,
                n if n >= lowest_success => CheckResult::Success,
                _ => CheckResult::Failure,
            };
            assert_eq!(outcome.result, expected, "natural {natural} vs {check:?}");
        }
    }

    #[test]
    fn natural_20_and_1_override_the_numbers() {
        // Impossible without a crit, and guaranteed except for a fumble.
        assert_succeeds_from(Check::static_tn(40, 0), 21);
        assert_succeeds_from(Check::static_tn(2, 10), 2);
    }

    #[test]
    fn plain_static_check() {
        // TN 12 with +3: needs a natural 9.
        assert_succeeds_from(Check::static_tn(12, 3), 9);
    }

    #[test]
    fn mastery_example_from_rulebook() {
        // Average lock, DEX +1 → TN 12, +2 Mastery: "only needs to roll a 9 or higher" (p.45).
        let check = Check::dynamic(TnTable::Standard, Difficulty::Average, 1).with_mastery(2);
        assert_succeeds_from(check, 9);
    }

    #[test]
    fn burden_example_from_rulebook() {
        // Average DR, modifier +2 → TN 13, -1 Burden: "must roll 12 or higher" (p.45).
        let check = Check::dynamic(TnTable::Standard, Difficulty::Average, 2).with_burden(1);
        assert_succeeds_from(check, 12);
    }

    #[test]
    fn total_and_margin_for_every_face() {
        let check = Check::static_tn(15, 2).with_mastery(3).with_burden(1);
        for natural in 1..=20 {
            let outcome = check.resolve(natural);
            assert_eq!(outcome.total, i16::from(natural) + 4);
            assert_eq!(outcome.margin, outcome.total - 15);
        }
    }

    #[test]
    fn mastery_and_burden_accumulate() {
        let check = Check::static_tn(10, 0)
            .with_mastery(2)
            .with_mastery(1)
            .with_burden(2);
        assert_eq!((check.mastery, check.burden), (3, 2));
    }

    #[test]
    fn real_rolls_match_resolve_and_reach_every_result() {
        let check = Check::static_tn(11, 0);
        let mut seen = [false; 4];
        for _ in 0..ROLLS {
            let outcome = check.roll();
            assert_eq!(outcome.discarded, None);
            assert_eq!(outcome, check.resolve(outcome.natural));
            seen[outcome.result as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "some CheckResult never came up");
    }

    #[test]
    fn advantage_and_disadvantage_keep_the_right_die() {
        let advantage = Check::static_tn(11, 0).with_mode(RollMode::Advantage);
        let disadvantage = Check::static_tn(11, 0).with_mode(RollMode::Disadvantage);
        for _ in 0..ROLLS {
            let up = advantage.roll();
            assert!(up.natural >= up.discarded.unwrap());
            assert_eq!(up.result, advantage.resolve(up.natural).result);

            let down = disadvantage.roll();
            assert!(down.natural <= down.discarded.unwrap());
            assert_eq!(down.result, disadvantage.resolve(down.natural).result);
        }
    }
}
