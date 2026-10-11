use serde::{Deserialize, Serialize};

use super::Difficulty;

/// Which Dynamic Target Number table to use. This is a campaign setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TnTable {
    /// The default table (p.43). Every point of modifier raises the TN by one, so a
    /// character's Attributes never make a task easier, only Mastery and Burden do.
    #[default]
    Standard,
    /// The optional Scaling table (Appendix II, p.230). The TN rises more slowly than the
    /// modifier, so higher Attributes still make a noticeable difference.
    Scaling,
}

/// The lowest and highest total modifiers printed in both tables.
const MIN_MODIFIER: i8 = -5;
const MAX_MODIFIER: i8 = 10;

/// Standard table (p.43): the Very Easy TN for a +0 modifier.
const STANDARD_VERY_EASY_AT_ZERO: i8 = 7;

/// Scaling table (p.230): the Very Easy column for modifiers -5 to +10.
const SCALING_VERY_EASY: [u8; 16] = [5, 5, 6, 6, 7, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12];

/// In both tables each difficulty column is 2 higher than the one before it.
const COLUMN_STEP: u8 = 2;

impl TnTable {
    /// Looks up the Target Number for a task of `difficulty`, attempted by a character whose
    /// total modifier for the roll is `modifier` (p.42).
    ///
    /// Modifiers outside the printed range (-5 to +10) use the nearest printed row.
    pub fn target_number(self, difficulty: Difficulty, modifier: i8) -> u8 {
        let modifier = modifier.clamp(MIN_MODIFIER, MAX_MODIFIER);
        let very_easy = match self {
            TnTable::Standard => (STANDARD_VERY_EASY_AT_ZERO + modifier) as u8,
            TnTable::Scaling => SCALING_VERY_EASY[(modifier - MIN_MODIFIER) as usize],
        };
        very_easy + COLUMN_STEP * difficulty.index() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Difficulty::*;

    /// Rows transcribed from the rulebook: (modifier, [VeryEasy .. NearlyImpossible]).
    const STANDARD_ROWS: [(i8, [u8; 6]); 4] = [
        (-5, [2, 4, 6, 8, 10, 12]),
        (0, [7, 9, 11, 13, 15, 17]),
        (2, [9, 11, 13, 15, 17, 19]),
        (10, [17, 19, 21, 23, 25, 27]),
    ];
    const SCALING_ROWS: [(i8, [u8; 6]); 6] = [
        (-5, [5, 7, 9, 11, 13, 15]),
        (-2, [6, 8, 10, 12, 14, 16]),
        (1, [7, 9, 11, 13, 15, 17]),
        (4, [9, 11, 13, 15, 17, 19]),
        (7, [10, 12, 14, 16, 18, 20]),
        (10, [12, 14, 16, 18, 20, 22]),
    ];

    fn check_rows(table: TnTable, rows: &[(i8, [u8; 6])]) {
        for &(modifier, expected) in rows {
            for (difficulty, &tn) in Difficulty::ALL.iter().zip(&expected) {
                assert_eq!(
                    table.target_number(*difficulty, modifier),
                    tn,
                    "{table:?} {difficulty:?} at modifier {modifier}"
                );
            }
        }
    }

    #[test]
    fn standard_matches_rulebook_rows() {
        check_rows(TnTable::Standard, &STANDARD_ROWS);
    }

    #[test]
    fn scaling_matches_rulebook_rows() {
        check_rows(TnTable::Scaling, &SCALING_ROWS);
    }

    #[test]
    fn rulebook_examples() {
        // Isabella: DEX +1, Hard jump → 14 (p.43). Lockpicking: +1, Average → 12 (p.45).
        assert_eq!(TnTable::Standard.target_number(Hard, 1), 14);
        assert_eq!(TnTable::Standard.target_number(Average, 1), 12);
        // Lora vs a scavenger's Average Perception with +2 → 13 (p.44).
        assert_eq!(TnTable::Standard.target_number(Average, 2), 13);
    }

    #[test]
    fn modifiers_off_the_table_use_the_edge_rows() {
        for table in [TnTable::Standard, TnTable::Scaling] {
            assert_eq!(
                table.target_number(Average, 25),
                table.target_number(Average, 10)
            );
            assert_eq!(
                table.target_number(Average, -9),
                table.target_number(Average, -5)
            );
        }
    }

    #[test]
    fn standard_is_the_default() {
        assert_eq!(TnTable::default(), TnTable::Standard);
    }
}
