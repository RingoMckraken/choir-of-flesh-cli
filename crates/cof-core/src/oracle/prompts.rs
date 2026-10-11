use std::fmt;

use super::data::{SurpriseRow, TABLES};
use crate::dice::Die;

/// An Action + Theme pair to interpret (p.86–88).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionTheme {
    pub action: (u8, &'static str),
    pub theme: (u8, &'static str),
}

/// A Surprise/Unknown Event (2D8, p.85–86).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurpriseEvent {
    pub row_roll: u8,
    pub column_roll: u8,
    pub row: &'static SurpriseRow,
    pub flavor: &'static str,
}

/// "Is there anything of interest?" (D10), and its Significance (D8) if so (p.89).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interest {
    pub roll: u8,
    pub significance: Option<(u8, &'static str)>,
}

/// Random Action Resolution (D20, ±5 for circumstances, p.89).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionResolution {
    pub roll: u8,
    pub modifier: i8,
    pub total: u8,
    pub result: &'static str,
}

pub fn action_theme() -> ActionTheme {
    let action = TABLES.actions.roll();
    let theme = TABLES.themes.roll();
    ActionTheme {
        action: (action.roll, action.value),
        theme: (theme.roll, theme.value),
    }
}

pub fn surprise_event() -> SurpriseEvent {
    let row = TABLES.surprise.roll();
    let column_roll = Die::D8.roll();
    SurpriseEvent {
        row_roll: row.roll,
        column_roll,
        row: row.value,
        flavor: &row.value.flavors[usize::from(column_roll - 1) / 2],
    }
}

pub fn anything_of_interest() -> Interest {
    let roll = Die::D10.roll();
    let significance = (roll >= 5).then(|| {
        let s = TABLES.significance.roll();
        (s.roll, s.value.as_str())
    });
    Interest { roll, significance }
}

/// The modifier is clamped to ±5, and the total to the D20's range.
pub fn resolve_action(modifier: i8) -> ActionResolution {
    let modifier = modifier.clamp(-5, 5);
    let roll = Die::D20.roll();
    let total = (roll as i8 + modifier).clamp(1, 20) as u8;
    ActionResolution {
        roll,
        modifier,
        total,
        result: TABLES.resolution.lookup(total).expect("1–20 is covered"),
    }
}

impl fmt::Display for ActionTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} · {}\n(Action D100 = {}, Theme D100 = {})",
            self.action.1, self.theme.1, self.action.0, self.theme.0
        )
    }
}

impl fmt::Display for SurpriseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} (D8 = {}, D8 = {})\n{}",
            self.row.kind, self.flavor, self.row_roll, self.column_roll, self.row.description
        )
    }
}

impl fmt::Display for Interest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.significance {
            None => write!(
                f,
                "D10 = {}: No. Things are exactly as they appear to be.",
                self.roll
            ),
            Some((roll, text)) => write!(
                f,
                "D10 = {}: Yes!\nSignificance (D8 = {roll}): {text}",
                self.roll
            ),
        }
    }
}

impl fmt::Display for ActionResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "D20 = {}", self.roll)?;
        if self.modifier != 0 {
            write!(f, " {:+} = {}", self.modifier, self.total)?;
        }
        write!(f, " → {}", self.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn surprise_columns_follow_the_d8_pairs() {
        let mut seen = HashSet::new();
        for _ in 0..10_000 {
            let event = surprise_event();
            let column = usize::from(event.column_roll - 1) / 2;
            assert_eq!(event.flavor, event.row.flavors[column]);
            seen.insert((event.row_roll, event.column_roll));
        }
        assert_eq!(seen.len(), 64, "every 2D8 combination should come up");
    }

    #[test]
    fn interest_needs_five_or_more() {
        let mut seen = [false; 10];
        for _ in 0..5_000 {
            let interest = anything_of_interest();
            assert_eq!(interest.significance.is_some(), interest.roll >= 5);
            seen[usize::from(interest.roll) - 1] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn resolution_applies_and_clamps_the_modifier() {
        for modifier in [-9, -5, 0, 3, 5, 9] {
            for _ in 0..2_000 {
                let r = resolve_action(modifier);
                assert_eq!(r.modifier, modifier.clamp(-5, 5));
                assert_eq!(r.total, (r.roll as i8 + r.modifier).clamp(1, 20) as u8);
                assert_eq!(
                    Some(r.result),
                    TABLES.resolution.lookup(r.total).map(String::as_str)
                );
            }
        }
    }

    #[test]
    fn action_theme_reaches_every_entry() {
        let mut actions = HashSet::new();
        let mut themes = HashSet::new();
        for _ in 0..20_000 {
            let pair = action_theme();
            actions.insert(pair.action.0);
            themes.insert(pair.theme.0);
        }
        assert_eq!((actions.len(), themes.len()), (100, 100));
    }
}
