use rand::RngExt;

/// The dice used by Choir of Flesh (p.9).
///
/// A D100 is a percentile roll made with two D10s, and a result of "00" counts as 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Die {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

/// How many dice to roll for a single result (p.43–44).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RollMode {
    #[default]
    Normal,
    /// Roll twice and keep the higher result.
    Advantage,
    /// Roll twice and keep the lower result.
    Disadvantage,
}

/// The result of rolling a die under a [`RollMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rolled {
    /// The result that counts.
    pub kept: u8,
    /// The die that was rolled and thrown away (Advantage/Disadvantage only).
    pub discarded: Option<u8>,
}

impl Die {
    /// Every die, smallest first.
    pub const ALL: [Die; 7] = [
        Die::D4,
        Die::D6,
        Die::D8,
        Die::D10,
        Die::D12,
        Die::D20,
        Die::D100,
    ];

    /// The number of faces on the die.
    pub fn sides(self) -> u8 {
        match self {
            Die::D4 => 4,
            Die::D6 => 6,
            Die::D8 => 8,
            Die::D10 => 10,
            Die::D12 => 12,
            Die::D20 => 20,
            Die::D100 => 100,
        }
    }

    /// Rolls the die once, giving a result from 1 to [`Die::sides`].
    pub fn roll(self) -> u8 {
        rand::rng().random_range(1..=self.sides())
    }

    /// Rolls the die using the given [`RollMode`].
    pub fn roll_with(self, mode: RollMode) -> Rolled {
        let first = self.roll();
        let pick = match mode {
            RollMode::Normal => {
                return Rolled {
                    kept: first,
                    discarded: None,
                };
            }
            RollMode::Advantage => u8::max,
            RollMode::Disadvantage => u8::min,
        };
        let second = self.roll();
        let kept = pick(first, second);
        let discarded = if kept == first { second } else { first };
        Rolled {
            kept,
            discarded: Some(discarded),
        }
    }

    /// Rolls `count` dice and adds them together, as in "3D6".
    pub fn roll_sum(self, count: u8) -> u16 {
        (0..count).map(|_| u16::from(self.roll())).sum()
    }

    /// The next die down the Usage Die chain (p.46): D20 → D12 → D10 → D8 → D6 → D4.
    ///
    /// Returns `None` for the D4 (the bottom of the chain) and the D100 (not part of it).
    pub fn step_down(self) -> Option<Die> {
        match self {
            Die::D20 => Some(Die::D12),
            Die::D12 => Some(Die::D10),
            Die::D10 => Some(Die::D8),
            Die::D8 => Some(Die::D6),
            Die::D6 => Some(Die::D4),
            Die::D4 | Die::D100 => None,
        }
    }

    /// The next die up the Usage Die chain, for restocking (e.g. buying arrows, p.214).
    ///
    /// Returns `None` for the D20 (the top of the chain) and the D100 (not part of it).
    pub fn step_up(self) -> Option<Die> {
        match self {
            Die::D4 => Some(Die::D6),
            Die::D6 => Some(Die::D8),
            Die::D8 => Some(Die::D10),
            Die::D10 => Some(Die::D12),
            Die::D12 => Some(Die::D20),
            Die::D20 | Die::D100 => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Enough rolls that every face of every die shows up (missing a D100 face in 20,000
    /// rolls has a probability far below 1e-80).
    const ROLLS: usize = 20_000;

    #[test]
    fn every_face_comes_up_and_nothing_else() {
        for die in Die::ALL {
            let mut seen = vec![false; usize::from(die.sides())];
            for _ in 0..ROLLS {
                let result = die.roll();
                assert!(
                    (1..=die.sides()).contains(&result),
                    "{die:?} rolled {result}"
                );
                seen[usize::from(result) - 1] = true;
            }
            assert!(seen.iter().all(|&s| s), "some {die:?} face never came up");
        }
    }

    #[test]
    fn normal_mode_rolls_one_die() {
        for _ in 0..ROLLS {
            let rolled = Die::D20.roll_with(RollMode::Normal);
            assert_eq!(rolled.discarded, None);
            assert!((1..=20).contains(&rolled.kept));
        }
    }

    #[test]
    fn advantage_always_keeps_the_higher_die() {
        let mut saw_20 = false;
        for _ in 0..ROLLS {
            let rolled = Die::D20.roll_with(RollMode::Advantage);
            let other = rolled.discarded.expect("advantage rolls two dice");
            assert!(rolled.kept >= other);
            saw_20 |= rolled.kept == 20;
        }
        assert!(saw_20);
    }

    #[test]
    fn disadvantage_always_keeps_the_lower_die() {
        let mut saw_1 = false;
        for _ in 0..ROLLS {
            let rolled = Die::D20.roll_with(RollMode::Disadvantage);
            let other = rolled.discarded.expect("disadvantage rolls two dice");
            assert!(rolled.kept <= other);
            saw_1 |= rolled.kept == 1;
        }
        assert!(saw_1);
    }

    #[test]
    fn roll_sum_covers_its_whole_range() {
        // 3D6 can total 3..=18, and every total should come up.
        let mut seen = [false; 19];
        for _ in 0..ROLLS {
            let total = Die::D6.roll_sum(3);
            assert!((3..=18).contains(&total), "3D6 rolled {total}");
            seen[usize::from(total)] = true;
        }
        assert!(seen[3..].iter().all(|&s| s));
        assert_eq!(Die::D6.roll_sum(0), 0);
    }

    #[test]
    fn step_down_chain() {
        assert_eq!(Die::D20.step_down(), Some(Die::D12));
        assert_eq!(Die::D12.step_down(), Some(Die::D10));
        assert_eq!(Die::D10.step_down(), Some(Die::D8));
        assert_eq!(Die::D8.step_down(), Some(Die::D6));
        assert_eq!(Die::D6.step_down(), Some(Die::D4));
        assert_eq!(Die::D4.step_down(), None);
        assert_eq!(Die::D100.step_down(), None);
    }

    #[test]
    fn step_up_reverses_step_down() {
        for die in [Die::D4, Die::D6, Die::D8, Die::D10, Die::D12] {
            assert_eq!(die.step_up().and_then(Die::step_down), Some(die));
        }
        assert_eq!(Die::D20.step_up(), None);
        assert_eq!(Die::D100.step_up(), None);
    }
}
