use serde::{Deserialize, Serialize};

const HUMANITY_MAX_DEFAULT: u8 = 20;

/// Humanity (HUM, p.46): how much of a PC remains uncorrupted. Starts at 20, and at 0 the
/// character is lost and becomes an NPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Humanity {
    current: u8,
    max: u8,
}

impl Default for Humanity {
    fn default() -> Self {
        Self::new()
    }
}

impl Humanity {
    pub fn new() -> Humanity {
        Self::new_with_max(HUMANITY_MAX_DEFAULT)
    }

    /// For a raised maximum, e.g. the Empathic skill (+2, so 22) (p.18).
    pub fn new_with_max(max: u8) -> Humanity {
        Humanity { current: max, max }
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn max(&self) -> u8 {
        self.max
    }

    pub fn lose(&mut self, amount: u8) {
        self.current = self.current.saturating_sub(amount);
    }

    /// Adds Humanity without going over the maximum.
    pub fn regain(&mut self, amount: u8) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }

    /// Changes the maximum, e.g. when the Empathic skill is learned after creation. Only the
    /// cap moves; current Humanity is clamped if the new maximum is lower.
    pub fn set_max(&mut self, max: u8) {
        self.max = max;
        self.current = self.current.min(max);
    }

    pub fn is_lost(&self) -> bool {
        self.current == 0
    }

    /// The Effects of Low Humanity table (p.47) for the current score.
    pub fn effect(&self) -> &'static str {
        match self.current {
            0 => "You become an NPC: a Penitent or Unmade.",
            1..=4 => {
                "Animals flee when you approach them. Children cry at your touch. You crave the Flesh/Choir."
            }
            5..=9 => {
                "-2 Burden to social checks. You avoid humans, as they remind you of what you've lost."
            }
            10..=14 => {
                "-1 Burden to social checks. The Choir's song soothes you, creatures of the Flesh don't look so abhorrent to you."
            }
            _ => "None. You suffer nightmares, but no penalties.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_humanity_effects_follow_the_table() {
        let mut h = Humanity::new_with_max(22);
        let mut seen = std::collections::HashSet::new();
        for score in (0..=22).rev() {
            h.set_max(score);
            assert_eq!(h.current(), score);
            let expected = match score {
                0 => "You become",
                1..=4 => "Animals",
                5..=9 => "-2 Burden",
                10..=14 => "-1 Burden",
                _ => "None.",
            };
            assert!(h.effect().starts_with(expected), "{score}");
            seen.insert(h.effect());
        }
        assert_eq!(seen.len(), 5);
    }

    #[test]
    fn starts_at_20() {
        let h = Humanity::new();
        assert_eq!((h.current(), h.max()), (20, 20));
    }

    #[test]
    fn loss_reduces_current() {
        let mut h = Humanity::new();
        h.lose(3);
        assert_eq!(h.current(), 17);
    }

    #[test]
    fn does_not_go_below_zero() {
        let mut h = Humanity::new();
        h.lose(100);
        assert_eq!(h.current(), 0);
    }

    #[test]
    fn regain_does_not_exceed_max() {
        let mut h = Humanity::new();
        h.lose(5);
        h.regain(100);
        assert_eq!(h.current(), h.max());
    }

    #[test]
    fn is_lost_at_zero() {
        let mut h = Humanity::new();
        assert!(!h.is_lost());
        h.lose(20);
        assert!(h.is_lost());
    }

    #[test]
    fn set_max_moves_only_the_cap() {
        let mut h = Humanity::new();
        h.lose(4);
        h.set_max(22);
        assert_eq!((h.current(), h.max()), (16, 22));
        h.set_max(10);
        assert_eq!((h.current(), h.max()), (10, 10));
    }

    #[test]
    fn empathic_raises_max_to_22() {
        let h = Humanity::new_with_max(22);
        assert_eq!((h.current(), h.max()), (22, 22));
    }

    #[test]
    fn empathic_regain_reaches_22() {
        // Regression: regain must cap at self.max, not the default of 20.
        let mut h = Humanity::new_with_max(22);
        h.lose(10);
        assert_eq!(h.current(), 12);
        h.regain(100);
        assert_eq!(h.current(), 22);
    }
}
