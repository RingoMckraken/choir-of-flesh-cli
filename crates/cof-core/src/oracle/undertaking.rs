//! The Undertaking generator (p.75–83): a quest skeleton built from four D20 tables.

use std::fmt;

use super::data::{TABLES, UndertakingEntry};
use crate::tables::RollTable;

/// One of the four Undertaking tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndertakingStep {
    Hook,
    Development,
    Tension,
    Climax,
}

impl UndertakingStep {
    pub const ALL: [UndertakingStep; 4] = [
        UndertakingStep::Hook,
        UndertakingStep::Development,
        UndertakingStep::Tension,
        UndertakingStep::Climax,
    ];

    /// The recommended structure (p.75): Hook → Development → Tension → Development →
    /// Tension → Development → Climax.
    pub const ARC: [UndertakingStep; 7] = [
        UndertakingStep::Hook,
        UndertakingStep::Development,
        UndertakingStep::Tension,
        UndertakingStep::Development,
        UndertakingStep::Tension,
        UndertakingStep::Development,
        UndertakingStep::Climax,
    ];

    pub fn name(self) -> &'static str {
        match self {
            UndertakingStep::Hook => "Hook",
            UndertakingStep::Development => "Development",
            UndertakingStep::Tension => "Tension",
            UndertakingStep::Climax => "Climax",
        }
    }

    fn table(self) -> &'static RollTable<UndertakingEntry> {
        match self {
            UndertakingStep::Hook => &TABLES.hooks,
            UndertakingStep::Development => &TABLES.developments,
            UndertakingStep::Tension => &TABLES.tensions,
            UndertakingStep::Climax => &TABLES.climaxes,
        }
    }
}

/// A single roll on one of the Undertaking tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndertakingRoll {
    pub step: UndertakingStep,
    pub roll: u8,
    pub entry: &'static UndertakingEntry,
}

/// A whole Undertaking, one roll per step of [`UndertakingStep::ARC`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undertaking {
    pub steps: Vec<UndertakingRoll>,
}

/// Rolls one step, for when the story needs its next twist.
pub fn undertaking_step(step: UndertakingStep) -> UndertakingRoll {
    let roll = step.table().roll();
    UndertakingRoll {
        step,
        roll: roll.roll,
        entry: roll.value,
    }
}

/// Rolls a full arc. The three Developments and two Tensions are rerolled until they differ:
/// meeting "The Rival Party" twice in one story would be a dud.
pub fn undertaking() -> Undertaking {
    let mut steps: Vec<UndertakingRoll> = Vec::with_capacity(UndertakingStep::ARC.len());
    for step in UndertakingStep::ARC {
        let fresh = loop {
            let candidate = undertaking_step(step);
            if !steps
                .iter()
                .any(|s| (s.step, s.roll) == (candidate.step, candidate.roll))
            {
                break candidate;
            }
        };
        steps.push(fresh);
    }
    Undertaking { steps }
}

impl fmt::Display for UndertakingRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (D20 = {}): {}\n{}",
            self.step.name(),
            self.roll,
            self.entry.title,
            self.entry.text
        )
    }
}

/// The outline comes first so the whole shape fits on screen; the write-ups follow.
impl fmt::Display for Undertaking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.steps {
            writeln!(f, "{} (D20 = {}): {}", s.step.name(), s.roll, s.entry.title)?;
        }
        for s in &self.steps {
            write!(f, "\n{}: {}", s.entry.title, s.entry.text)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn single_steps_reach_every_entry_of_their_table() {
        for step in UndertakingStep::ALL {
            let mut seen = HashSet::new();
            for _ in 0..2_000 {
                let r = undertaking_step(step);
                assert_eq!(r.step, step);
                assert_eq!(Some(r.entry), step.table().lookup(r.roll));
                seen.insert(r.roll);
            }
            assert_eq!(seen.len(), 20, "{step:?}");
        }
    }

    #[test]
    fn an_arc_follows_the_structure_without_repeats() {
        let mut seen: HashMap<UndertakingStep, HashSet<u8>> = HashMap::new();
        for _ in 0..3_000 {
            let arc = undertaking();
            let shape: Vec<_> = arc.steps.iter().map(|s| s.step).collect();
            assert_eq!(shape, UndertakingStep::ARC);
            // Compare (table, roll), not titles: "The Cannibals" is both Hook 3 and Tension 6.
            let picks: HashSet<_> = arc.steps.iter().map(|s| (s.step, s.roll)).collect();
            assert_eq!(picks.len(), arc.steps.len(), "no entry repeats in one arc");
            for s in &arc.steps {
                assert_eq!(Some(s.entry), s.step.table().lookup(s.roll));
                seen.entry(s.step).or_default().insert(s.roll);
            }
        }
        for step in UndertakingStep::ALL {
            assert_eq!(seen[&step].len(), 20, "{step:?}");
        }
    }

    #[test]
    fn the_arc_text_leads_with_the_outline() {
        let arc = undertaking();
        let text = arc.to_string();
        let first: Vec<_> = text.lines().take(7).collect();
        assert!(first[0].starts_with("Hook (D20 = "));
        assert!(first[6].starts_with("Climax (D20 = "));
        assert!(text.ends_with(&arc.steps[6].entry.text));
    }
}
