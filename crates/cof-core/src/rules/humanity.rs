const HUMANITY_MAX_DEFAULT: u8 = 20;

/// Humanity (HUM, p.46): how much of a PC remains uncorrupted. Starts at 20, and at 0 the
/// character is lost and becomes an NPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn is_lost(&self) -> bool {
        self.current == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
