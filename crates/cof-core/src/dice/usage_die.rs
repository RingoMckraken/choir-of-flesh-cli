use serde::{Deserialize, Serialize};

use super::Die;

/// A Usage Die (p.46, p.90): tracks a dwindling resource or an approaching event with a
/// single die instead of a counter.
///
/// Roll it when circumstances call for it. A 1 or 2 steps it down the chain
/// (D20 → D12 → D10 → D8 → D6 → D4). A 1 or 2 on the D4 means it's **depleted**: the torch
/// goes out, the quiver is empty, the event happens. Written in the book as "UD8", etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageDie {
    die: Die,
}

/// What happened when a Usage Die was rolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageOutcome {
    /// 3 or higher: nothing changes.
    Holds,
    /// 1 or 2: the die shrinks to the next size.
    SteppedDown { from: Die, to: Die },
    /// 1 or 2 on the D4: the resource is gone or the event triggers.
    Depleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageRoll {
    pub die: Die,
    pub roll: u8,
    pub outcome: UsageOutcome,
}

impl UsageDie {
    /// Creates a Usage Die of the given size. Returns `None` for the D100, which isn't part
    /// of the Usage Die chain.
    pub fn new(die: Die) -> Option<Self> {
        (die != Die::D100).then_some(Self { die })
    }

    /// The current size of the die.
    pub fn die(self) -> Die {
        self.die
    }

    /// Rolls the die and applies the result.
    ///
    /// After [`UsageOutcome::Depleted`] the die stays a D4. Discarding the depleted
    /// resource (or resolving the event) is up to the caller.
    pub fn roll(&mut self) -> UsageRoll {
        let roll = self.die.roll();
        self.apply(roll)
    }

    /// Applies a result the player rolled on the current die themselves.
    pub fn apply(&mut self, roll: u8) -> UsageRoll {
        let die = self.die;
        let outcome = if roll > 2 {
            UsageOutcome::Holds
        } else {
            match die.step_down() {
                Some(next) => {
                    self.die = next;
                    UsageOutcome::SteppedDown {
                        from: die,
                        to: next,
                    }
                }
                None => UsageOutcome::Depleted,
            }
        };
        UsageRoll { die, roll, outcome }
    }

    /// Steps the die one size up, but never above `cap` (e.g. a Quiver holds up to UD8,
    /// p.210). Returns `true` if the die grew.
    pub fn restock(&mut self, cap: Die) -> bool {
        match self.die.step_up() {
            Some(next) if next.sides() <= cap.sides() => {
                self.die = next;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ud(die: Die) -> UsageDie {
        UsageDie::new(die).unwrap()
    }

    #[test]
    fn d100_is_not_a_usage_die() {
        assert_eq!(UsageDie::new(Die::D100), None);
    }

    #[test]
    fn every_face_of_every_size_follows_the_rule() {
        for die in [Die::D4, Die::D6, Die::D8, Die::D10, Die::D12, Die::D20] {
            for face in 1..=die.sides() {
                let mut usage = ud(die);
                let result = usage.apply(face);
                let expected = match (face, die.step_down()) {
                    (3.., _) => UsageOutcome::Holds,
                    (_, Some(to)) => UsageOutcome::SteppedDown { from: die, to },
                    (_, None) => UsageOutcome::Depleted,
                };
                assert_eq!(result.outcome, expected, "{die:?} face {face}");
            }
        }
    }

    #[test]
    fn real_rolls_always_run_the_chain_to_depletion() {
        // A UD20 must always walk D20 → … → D4 and eventually deplete, and every roll
        // along the way must match the rule.
        for _ in 0..200 {
            let mut clock = ud(Die::D20);
            let mut steps = 0;
            loop {
                let before = clock.die();
                let result = clock.roll();
                assert!((1..=before.sides()).contains(&result.roll));
                match result.outcome {
                    UsageOutcome::Holds => assert!(result.roll >= 3),
                    UsageOutcome::SteppedDown { from, to } => {
                        assert!(result.roll <= 2);
                        assert_eq!((from, Some(to)), (before, before.step_down()));
                        steps += 1;
                    }
                    UsageOutcome::Depleted => {
                        assert_eq!(before, Die::D4);
                        assert!(result.roll <= 2);
                        break;
                    }
                }
            }
            assert_eq!(steps, 5, "D20 → D12 → D10 → D8 → D6 → D4 is five steps");
        }
    }

    #[test]
    fn restock_respects_cap() {
        let mut quiver = ud(Die::D6);
        assert!(quiver.restock(Die::D8));
        assert_eq!(quiver.die(), Die::D8);
        assert!(!quiver.restock(Die::D8));
        assert_eq!(quiver.die(), Die::D8);
    }
}
