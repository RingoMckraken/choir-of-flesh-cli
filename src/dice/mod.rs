use rand::RngExt;

#[derive(Debug, PartialEq)]
pub enum Die {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
}

impl Die {
    /// Defines the number of sides any given die has
    pub fn sides(&self) -> u8 {
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

    /// Rolls the die with a result from 1 to the max of side() inclusive
    pub fn roll(&self) -> u8 {
        rand::rng().random_range(1..=self.sides())
    }

    /// Advantage rolls two dice and takes the greater of the two
    pub fn roll_advantage(&self) -> u8 {
        let a = self.roll();
        let b = self.roll();
        a.max(b)
    }

    /// Disadvantage rolls two dice and takes the lower of the two
    pub fn roll_disadvantage(&self) -> u8 {
        let a = self.roll();
        let b = self.roll();
        a.min(b)
    }

    /// For use with usage die so if a usage die like a d20 rolls 1 or a 2 then
    /// we will step down to d12. If a usage die is a d4 and rolls a 1 or a 2,
    /// we will return None to signify that a consequence takes place like a
    /// torch going out or arrows in a quiver are depleted.
    pub fn step_down(&self) -> Option<Die> {
        match self {
            Die::D20 => Some(Die::D12),
            Die::D12 => Some(Die::D10),
            Die::D10 => Some(Die::D8),
            Die::D8 => Some(Die::D6),
            Die::D6 => Some(Die::D4),
            Die::D4 => None,
            Die::D100 => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_is_in_range() {
        let dice = [Die::D4, Die::D8, Die::D10, Die::D12, Die::D20, Die::D100];
        for die in &dice {
            for _ in 0..100 {
                let result = die.roll();
                assert!(
                    result >= 1 && result <= die.sides(),
                    "Die {:?} rolled {} which is out of range 1..={}",
                    die, result, die.sides()
                );
            }
        }
    }

    #[test]
    fn advantage_disadvantage_in_range() {
        for _ in 0..100 {
            let adv = Die::D20.roll_advantage();
            let dis = Die::D20.roll_disadvantage();
            assert!(adv >= 1 && adv <= 20);
            assert!(dis >= 1 && dis <= 20);
        }
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
}

