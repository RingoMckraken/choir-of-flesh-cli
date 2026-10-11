mod checks;
mod humanity;
mod anguish;

pub use checks::{Difficulty, CheckResult, CheckOutcome, roll_check, dynamic_tn, roll_dynamic_check};
pub use humanity::Humanity;
pub use anguish::{Anguish, BreakdownResult};


#[cfg(test)]
mod tests {
    use super::*;

    // --- Check Resolution Tests --- //
    #[test]
    fn natural_20_is_always_crit_success() {
        // We can't force a natural 20, but we can test the logic directly.
        // Roll many times -- over enough iterations we should see at least one
        // CriticalSuccess, proving the code path is reachable.
        let mut saw_crit = false;
        for _ in 0..1000 {
            let outcome = roll_check(0, 15);
            if outcome.result == CheckResult::CriticalSuccess {
                assert_eq!(outcome.roll, 20, "CriticalSuccess should only occur on a roll of 20");
                saw_crit = true;
            }
        }
        assert!(saw_crit, "After 1000 rolls, never saw a CriticalSuccess - something is wrong");
    }

    #[test]
    fn natural_1_is_always_crit_failure() {
        let mut saw_crit = false;
        for _ in 0..1000 {
            let outcome = roll_check(10, 5);
            if outcome.result == CheckResult::CriticalFailure {
                assert_eq!(outcome.roll, 1, "CriticalFailure should only occur on a roll of 1");
                saw_crit = true;
            }
        }
        assert!(saw_crit, "After 1000 rolls, never saw a CriticalFailure - something is wrong");
    }

    #[test]
    fn success_when_total_meets_tn() {
        // With modifier +10 and TN 11, any roll of 1+ succeeds (except natural 1 -> crit fail).
        // After 500 rolls we should have seen at least some successes.
        let mut saw_success = false;
        for _ in 0..500 {
            let outcome = roll_check(10, 11);
            if outcome.result == CheckResult::Success {
                saw_success = true;
                assert!(outcome.margin >= 0, "Success should have non-negative margin");
            }
        }
        assert!(saw_success);
    }

    #[test]
    fn failure_when_total_misses_tn() {
        // With modifier -10 and TN 19, almost every roll fails.
        let mut saw_failure = false;
        for _ in 0..500 {
            let outcome = roll_check(-10, 19);
            if outcome.result == CheckResult::Failure {
                saw_failure = true;
                assert!(outcome.margin < 0, "Failure should have negative margin");
            }
        }
        assert!(saw_failure);
    }

    #[test]
    fn margin_is_correct() {
        // We can't control the roll, but we can verify the margin formula.
        // Roll many times and check that margin == (roll + modifier) - tn for every outcome.
        for _ in 0..200 {
            let modifier: i8 = 3;
            let tn: u8 = 12;
            let outcome = roll_check(modifier, tn);
            let expected_margin = outcome.roll as i8 + modifier  - tn as i8;
            assert_eq!(outcome.margin, expected_margin,
                       "Margin mismatch: roll={}, modifier={}, tn={}", outcome.roll, modifier, tn);
        }
    }

    // -- Dynamic TN Tests --- /

    #[test]
    fn dynamic_tn_average_modifier_plus_1() {
        assert_eq!(dynamic_tn(Difficulty::Average, 1), 12);
    }

    #[test]
    fn dynamic_tn_average_modifier_plus_2() {
        assert_eq!(dynamic_tn(Difficulty::Average, 2), 13);
    }

    #[test]
    fn dynamic_tn_hard_modifier_plus_1() {
        assert_eq!(dynamic_tn(Difficulty::Hard, 1), 14);
    }

    #[test]
    fn dynamic_tn_hard_modifier_plus_2() {
        assert_eq!(dynamic_tn(Difficulty::Hard, 2), 15);
    }

    #[test]
    fn dynamic_tn_clamps_at_minimum() {
        // Very high modifier shouldn't produce a TN below 2
        assert!(dynamic_tn(Difficulty::Easy, 20) >= 2);
    }

    #[test]
    fn dynamic_tn_clamps_at_maximum() {
        // A very high modifier on Nearly Impossible (10 + 17 = 27) is the table ceiling.
        // Going beyond that (e.g. modifier 20) should still clamp to 27, not exceed it.
        assert_eq!(dynamic_tn(Difficulty::NearlyImpossible, 10), 27);
        assert!(dynamic_tn(Difficulty::NearlyImpossible, 20) <= 27);
    }

    // --- Humanity Tests --- //
    #[test]
    fn humanity_starts_at_20() {
        let h = Humanity::new();
        assert_eq!(h.current(), 20);
        assert_eq!(h.max(), 20);
    }

    #[test]
    fn humanity_loss_reduces_current() {
        let mut h = Humanity::new();
        h.lose(3);
        assert_eq!(h.current(), 17);
    }

    #[test]
    fn humanity_does_not_go_below_zero() {
        let mut h = Humanity::new();
        h.lose(100);
        assert_eq!(h.current(), 0);
    }

    #[test]
    fn humanity_regain_does_not_exceed_max() {
        let mut h = Humanity::new();
        h.lose(5);
        h.regain(100);
        assert_eq!(h.current(), h.max());
    }

    #[test]
    fn humanity_is_lost_at_zero() {
        let mut h = Humanity::new();
        assert!(!h.is_lost());
        h.lose(20);
        assert!(h.is_lost());
    }

    #[test]
    fn humanity_with_empathic_feat_has_max_22() {
        let h = Humanity::new_with_max(22);
        assert_eq!(h.current(), 22);
        assert_eq!(h.max(), 22);
    }

    #[test]
    fn humanity_empathic_regain_reaches_22() {
        // Regression test: regain must cap at self.max, not HUMANITY_MAX_DEFAULT.
        // An Empathic character who lost humanity should be able to recover all the
        // way back to 22 - if regain used the constant (20), they'd be stuck at 20.
        let mut h = Humanity::new_with_max(22);
        h.lose(10);
        assert_eq!(h.current(), 12);
        h.regain(100);
        assert_eq!(h.current(), 22);
    }
}