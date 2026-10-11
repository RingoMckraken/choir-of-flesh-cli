use serde::{Deserialize, Serialize};

use crate::dice::{Die, UsageDie, UsageOutcome, UsageRoll};

/// A named Usage Die the player is tracking: a torch, a quiver, a ticking clock (p.46, p.90).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageTracker {
    pub name: String,
    pub die: UsageDie,
    /// Set when the die is depleted. Cleared by restocking.
    pub depleted: bool,
}

impl UsageTracker {
    pub fn new(name: impl Into<String>, die: UsageDie) -> Self {
        Self {
            name: name.into(),
            die,
            depleted: false,
        }
    }

    pub fn roll(&mut self) -> UsageRoll {
        let roll = self.die.roll();
        if roll.outcome == UsageOutcome::Depleted {
            self.depleted = true;
        }
        roll
    }

    /// One size up (up to a D20), and no longer depleted.
    pub fn restock(&mut self) -> bool {
        if self.depleted {
            self.depleted = false;
            return true;
        }
        self.die.restock(Die::D20)
    }

    /// A journal line for a roll, e.g. "Torch (UD6) rolled 2: down to UD4."
    pub fn describe(&self, roll: &UsageRoll) -> String {
        let what = match roll.outcome {
            UsageOutcome::Holds => "holds".to_owned(),
            UsageOutcome::SteppedDown { to, .. } => format!("down to UD{}", to.sides()),
            UsageOutcome::Depleted => "DEPLETED!".to_owned(),
        };
        format!(
            "{} (UD{}) rolled {}: {what}",
            self.name,
            roll.die.sides(),
            roll.roll
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tracker_runs_down_to_depleted_and_restocks() {
        let mut torch = UsageTracker::new("Torch", UsageDie::new(Die::D4).unwrap());
        while !torch.depleted {
            let roll = torch.roll();
            assert!(torch.describe(&roll).starts_with("Torch (UD4) rolled"));
        }
        assert!(torch.restock());
        assert!(!torch.depleted);
        assert_eq!(torch.die.die(), Die::D4);
        assert!(torch.restock());
        assert_eq!(torch.die.die(), Die::D6);
    }

    #[test]
    fn survives_a_save() {
        let tracker = UsageTracker::new("Quiver", UsageDie::new(Die::D8).unwrap());
        let json = serde_json::to_string(&tracker).unwrap();
        assert_eq!(
            serde_json::from_str::<UsageTracker>(&json).unwrap(),
            tracker
        );
    }
}
