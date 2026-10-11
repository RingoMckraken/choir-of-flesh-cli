/// How hard a task is (p.42). Also used for creature stats, which are expressed as
/// difficulties that PCs roll against (p.175).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard,
    NearlyImpossible,
}

impl Difficulty {
    /// Every difficulty, easiest first.
    pub const ALL: [Difficulty; 6] = [
        Difficulty::VeryEasy,
        Difficulty::Easy,
        Difficulty::Average,
        Difficulty::Hard,
        Difficulty::VeryHard,
        Difficulty::NearlyImpossible,
    ];

    /// Position in [`Difficulty::ALL`]: 0 for Very Easy up to 5 for Nearly Impossible.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Moves the difficulty `steps` stages (positive = harder), stopping at either end.
    ///
    /// Covers "Shifting the TN" up or down (p.43) and creature stats moving "one stage"
    /// for Easy and Hard encounters (p.174).
    pub fn shifted(self, steps: i8) -> Difficulty {
        let index = (self.index() as i16 + i16::from(steps)).clamp(0, Self::ALL.len() as i16 - 1);
        Self::ALL[index as usize]
    }

    /// One stage harder (saturating).
    pub fn harder(self) -> Difficulty {
        self.shifted(1)
    }

    /// One stage easier (saturating).
    pub fn easier(self) -> Difficulty {
        self.shifted(-1)
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::VeryEasy => "Very Easy",
            Difficulty::Easy => "Easy",
            Difficulty::Average => "Average",
            Difficulty::Hard => "Hard",
            Difficulty::VeryHard => "Very Hard",
            Difficulty::NearlyImpossible => "Nearly Impossible",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifting_moves_by_stages() {
        assert_eq!(Difficulty::Hard.easier(), Difficulty::Average);
        assert_eq!(Difficulty::Average.harder(), Difficulty::Hard);
        assert_eq!(Difficulty::Easy.shifted(3), Difficulty::VeryHard);
    }

    #[test]
    fn shifting_saturates_at_the_ends() {
        assert_eq!(Difficulty::VeryEasy.easier(), Difficulty::VeryEasy);
        assert_eq!(
            Difficulty::NearlyImpossible.harder(),
            Difficulty::NearlyImpossible
        );
        assert_eq!(Difficulty::Average.shifted(-100), Difficulty::VeryEasy);
        assert_eq!(
            Difficulty::NearlyImpossible.shifted(i8::MAX),
            Difficulty::NearlyImpossible
        );
        assert_eq!(Difficulty::VeryEasy.shifted(i8::MIN), Difficulty::VeryEasy);
    }

    #[test]
    fn ordering_follows_hardness() {
        assert!(Difficulty::Easy < Difficulty::Hard);
        assert_eq!(
            Difficulty::ALL.iter().max(),
            Some(&Difficulty::NearlyImpossible)
        );
    }
}
