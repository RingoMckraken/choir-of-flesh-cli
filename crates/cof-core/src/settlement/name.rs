use crate::dice::Die;

/// Settlement Name Generator, prefix column (D20, p.36).
pub const NAME_PREFIXES: [&str; 20] = [
    "Last",
    "New",
    "Hope's",
    "Stone",
    "Quiet",
    "Ember",
    "Silent",
    "Broken",
    "Thorn",
    "Pilgrim's",
    "Mercy",
    "Faith's",
    "First",
    "Defiance",
    "Mourning's",
    "Dawn's",
    "Grey",
    "Saint's",
    "Hollow",
    "Lantern",
];

/// Settlement Name Generator, suffix column (D20, p.36).
pub const NAME_SUFFIXES: [&str; 20] = [
    "Rest",
    "Fall",
    "Watch",
    "Creek",
    "Gate",
    "Keep",
    "Cross",
    "Abbey",
    "Chapel",
    "Hold",
    "Sanctuary",
    "Bastion",
    "Refuge",
    "Post",
    "Landing",
    "Rock",
    "Ford",
    "Bridge",
    "End",
    "Ash",
];

/// The settlement name for a prefix roll and a suffix roll (each 1–20).
pub fn settlement_name(prefix_roll: u8, suffix_roll: u8) -> Option<String> {
    let prefix = NAME_PREFIXES.get(usize::from(prefix_roll).checked_sub(1)?)?;
    let suffix = NAME_SUFFIXES.get(usize::from(suffix_roll).checked_sub(1)?)?;
    Some(format!("{prefix} {suffix}"))
}

/// Rolls a D20 for each column, e.g. "Hope's Watch".
pub fn roll_name() -> String {
    settlement_name(Die::D20.roll(), Die::D20.roll()).expect("a D20 always rolls 1–20")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn rulebook_rows() {
        assert_eq!(settlement_name(1, 1).as_deref(), Some("Last Rest"));
        assert_eq!(settlement_name(3, 3).as_deref(), Some("Hope's Watch"));
        assert_eq!(settlement_name(20, 20).as_deref(), Some("Lantern Ash"));
        assert_eq!(
            settlement_name(10, 11).as_deref(),
            Some("Pilgrim's Sanctuary")
        );
    }

    #[test]
    fn rolls_outside_the_d20_are_rejected() {
        assert_eq!(settlement_name(0, 5), None);
        assert_eq!(settlement_name(5, 21), None);
    }

    #[test]
    fn rolling_reaches_every_prefix_and_suffix() {
        let mut prefixes = HashSet::new();
        let mut suffixes = HashSet::new();
        for _ in 0..5_000 {
            let name = roll_name();
            let (prefix, suffix) = name.split_once(' ').unwrap();
            assert!(NAME_PREFIXES.contains(&prefix) && NAME_SUFFIXES.contains(&suffix));
            prefixes.insert(prefix.to_owned());
            suffixes.insert(suffix.to_owned());
        }
        assert_eq!((prefixes.len(), suffixes.len()), (20, 20));
    }
}
