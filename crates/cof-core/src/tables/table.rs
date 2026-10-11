use std::ops::RangeInclusive;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::dice::Die;

/// A random table: a die and the entry for each range of results.
///
/// Construction guarantees the ranges cover every face of the die exactly once, in order,
/// so every roll finds exactly one entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RollTable<T> {
    name: String,
    die: Die,
    entries: Vec<(RangeInclusive<u8>, T)>,
}

/// A roll on a table: the die result and the entry it landed on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableRoll<'a, T> {
    pub roll: u8,
    pub value: &'a T,
}

#[derive(Debug, thiserror::Error)]
pub enum TableError {
    #[error("couldn't parse table: {0}")]
    Parse(#[from] ron::error::SpannedError),
    #[error("table \"{table}\": {problem}")]
    Invalid { table: String, problem: String },
}

/// The on-disk shape: entries are `(low, high, value)`.
#[derive(Deserialize)]
struct RawTable<T> {
    name: String,
    die: Die,
    entries: Vec<(u8, u8, T)>,
}

impl<T: DeserializeOwned> RollTable<T> {
    /// Parses and validates a table written in RON.
    pub fn from_ron(source: &str) -> Result<Self, TableError> {
        let raw: RawTable<T> = ron::from_str(source)?;
        Self::new(raw.name, raw.die, raw.entries)
    }
}

impl<T> RollTable<T> {
    /// Builds a table from `(low, high, value)` entries, checking that they cover
    /// 1..=sides with no gaps or overlaps.
    pub fn new(
        name: impl Into<String>,
        die: Die,
        entries: Vec<(u8, u8, T)>,
    ) -> Result<Self, TableError> {
        let name = name.into();
        let invalid = |problem: String| TableError::Invalid {
            table: name.clone(),
            problem,
        };
        let mut next = 1u16;
        for (low, high, _) in &entries {
            if u16::from(*low) != next {
                return Err(invalid(format!(
                    "expected an entry starting at {next}, found {low}"
                )));
            }
            if high < low {
                return Err(invalid(format!("range {low}-{high} runs backwards")));
            }
            next = u16::from(*high) + 1;
        }
        if next != u16::from(die.sides()) + 1 {
            return Err(invalid(format!(
                "entries end at {} but a {die:?} goes up to {}",
                next - 1,
                die.sides()
            )));
        }
        let entries = entries
            .into_iter()
            .map(|(low, high, value)| (low..=high, value))
            .collect();
        Ok(Self { name, die, entries })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn die(&self) -> Die {
        self.die
    }

    /// Every entry with its range, in table order.
    pub fn entries(&self) -> impl Iterator<Item = (&RangeInclusive<u8>, &T)> {
        self.entries.iter().map(|(range, value)| (range, value))
    }

    /// The entry for a result the player rolled themselves.
    pub fn lookup(&self, roll: u8) -> Option<&T> {
        self.entries
            .iter()
            .find(|(range, _)| range.contains(&roll))
            .map(|(_, value)| value)
    }

    /// Rolls the table's die and returns the matching entry.
    pub fn roll(&self) -> TableRoll<'_, T> {
        let roll = self.die.roll();
        TableRoll {
            roll,
            value: self
                .lookup(roll)
                .expect("validated tables cover every face"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AGES: &str = r#"(
        name: "Age",
        die: D6,
        entries: [(1, 2, "Young"), (3, 4, "Middle-aged"), (5, 6, "Elderly")],
    )"#;

    #[test]
    fn parses_and_looks_up_every_face() {
        let table: RollTable<String> = RollTable::from_ron(AGES).unwrap();
        assert_eq!((table.name(), table.die()), ("Age", Die::D6));
        let expected = [
            "Young",
            "Young",
            "Middle-aged",
            "Middle-aged",
            "Elderly",
            "Elderly",
        ];
        for (face, want) in (1..=6).zip(expected) {
            assert_eq!(table.lookup(face).map(String::as_str), Some(want));
        }
        assert_eq!(table.lookup(0), None);
        assert_eq!(table.lookup(7), None);
    }

    #[test]
    fn rolling_reaches_every_entry() {
        let table: RollTable<String> = RollTable::from_ron(AGES).unwrap();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1_000 {
            let rolled = table.roll();
            assert_eq!(Some(rolled.value), table.lookup(rolled.roll));
            seen.insert(rolled.value.clone());
        }
        assert_eq!(seen.len(), 3);
    }

    #[test]
    fn gaps_overlaps_and_short_tables_are_rejected() {
        let gap = RollTable::new("t", Die::D6, vec![(1, 2, ()), (4, 6, ())]);
        let overlap = RollTable::new("t", Die::D6, vec![(1, 3, ()), (3, 6, ())]);
        let short = RollTable::new("t", Die::D6, vec![(1, 5, ())]);
        let backwards = RollTable::new("t", Die::D6, vec![(1, 6, ()), (7, 6, ())]);
        for result in [gap, overlap, short, backwards] {
            assert!(matches!(result, Err(TableError::Invalid { .. })));
        }
    }

    #[test]
    fn bad_ron_is_a_parse_error() {
        let result: Result<RollTable<String>, _> = RollTable::from_ron("(name: ");
        assert!(matches!(result, Err(TableError::Parse(_))));
    }
}
