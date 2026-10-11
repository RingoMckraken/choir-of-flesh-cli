use std::fmt;
use std::str::FromStr;

use super::Die;

/// A dice expression like `2d6+1`, `d20`, or `3D8-2`: up to 20 of one die type plus a flat
/// modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiceExpr {
    pub count: u8,
    pub die: Die,
    pub modifier: i16,
}

/// The result of rolling a [`DiceExpr`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceTotal {
    pub expr: DiceExpr,
    pub rolls: Vec<u8>,
    pub total: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseDiceError {
    #[error("write dice like 2d6+1, d20, or 3d8-2")]
    Format,
    #[error("roll between 1 and {max} dice at a time", max = DiceExpr::MAX_DICE)]
    Count,
    #[error("there's no d{0} in this game (use d4, d6, d8, d10, d12, d20, or d100)")]
    Sides(String),
}

impl DiceExpr {
    pub const MAX_DICE: u8 = 20;

    pub fn roll(&self) -> DiceTotal {
        let rolls: Vec<u8> = (0..self.count).map(|_| self.die.roll()).collect();
        let total = rolls.iter().map(|&r| i32::from(r)).sum::<i32>() + i32::from(self.modifier);
        DiceTotal {
            expr: *self,
            rolls,
            total,
        }
    }

    /// The lowest and highest possible totals.
    pub fn range(&self) -> (i32, i32) {
        let count = i32::from(self.count);
        let modifier = i32::from(self.modifier);
        (
            count + modifier,
            count * i32::from(self.die.sides()) + modifier,
        )
    }
}

impl FromStr for DiceExpr {
    type Err = ParseDiceError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text: String = text
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_lowercase();
        let (count, rest) = text.split_once('d').ok_or(ParseDiceError::Format)?;
        let count = if count.is_empty() {
            1
        } else {
            count.parse::<u8>().map_err(|_| ParseDiceError::Count)?
        };
        if count == 0 || count > Self::MAX_DICE {
            return Err(ParseDiceError::Count);
        }

        let split = rest.find(['+', '-']).unwrap_or(rest.len());
        let (sides, modifier) = rest.split_at(split);
        let die = sides
            .parse::<u8>()
            .ok()
            .and_then(Die::from_sides)
            .ok_or_else(|| ParseDiceError::Sides(sides.to_owned()))?;
        let modifier = if modifier.is_empty() {
            0
        } else {
            let value: i16 = modifier[1..].parse().map_err(|_| ParseDiceError::Format)?;
            if modifier.starts_with('-') {
                -value
            } else {
                value
            }
        };
        Ok(Self {
            count,
            die,
            modifier,
        })
    }
}

impl fmt::Display for DiceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}D{}", self.count, self.die.sides())?;
        match self.modifier {
            0 => Ok(()),
            m if m > 0 => write!(f, "+{m}"),
            m => write!(f, "{m}"),
        }
    }
}

impl fmt::Display for DiceTotal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rolls: Vec<String> = self.rolls.iter().map(u8::to_string).collect();
        write!(f, "{}: [{}]", self.expr, rolls.join(", "))?;
        match self.expr.modifier {
            0 => {}
            m if m > 0 => write!(f, " + {m}")?,
            m => write!(f, " - {}", -m)?,
        }
        write!(f, " = {}", self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> DiceExpr {
        text.parse().unwrap()
    }

    #[test]
    fn parses_common_forms() {
        assert_eq!(
            parse("d20"),
            DiceExpr {
                count: 1,
                die: Die::D20,
                modifier: 0
            }
        );
        assert_eq!(
            parse("2d6+1"),
            DiceExpr {
                count: 2,
                die: Die::D6,
                modifier: 1
            }
        );
        assert_eq!(
            parse(" 3D8 - 2 "),
            DiceExpr {
                count: 3,
                die: Die::D8,
                modifier: -2
            }
        );
        assert_eq!(
            parse("d100"),
            DiceExpr {
                count: 1,
                die: Die::D100,
                modifier: 0
            }
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!("".parse::<DiceExpr>(), Err(ParseDiceError::Format));
        assert_eq!("2x6".parse::<DiceExpr>(), Err(ParseDiceError::Format));
        assert_eq!("0d6".parse::<DiceExpr>(), Err(ParseDiceError::Count));
        assert_eq!("21d6".parse::<DiceExpr>(), Err(ParseDiceError::Count));
        assert_eq!(
            "d7".parse::<DiceExpr>(),
            Err(ParseDiceError::Sides("7".into()))
        );
        assert_eq!("d6+x".parse::<DiceExpr>(), Err(ParseDiceError::Format));
    }

    #[test]
    fn display_round_trips() {
        for text in ["1D20", "2D6+1", "3D8-2", "1D100"] {
            assert_eq!(parse(text).to_string(), text);
        }
    }

    #[test]
    fn rolls_cover_the_whole_range() {
        let expr = parse("2d4+1");
        let (low, high) = expr.range();
        assert_eq!((low, high), (3, 9));
        let mut seen = std::collections::HashSet::new();
        for _ in 0..5_000 {
            let result = expr.roll();
            assert_eq!(result.rolls.len(), 2);
            let sum: i32 = result.rolls.iter().map(|&r| i32::from(r)).sum();
            assert_eq!(result.total, sum + 1);
            assert!((low..=high).contains(&result.total));
            seen.insert(result.total);
        }
        assert_eq!(seen.len(), 7, "every total from 3 to 9 should come up");
    }

    #[test]
    fn total_display() {
        let total = DiceTotal {
            expr: parse("2d6-1"),
            rolls: vec![3, 5],
            total: 7,
        };
        assert_eq!(total.to_string(), "2D6-1: [3, 5] - 1 = 7");
    }
}
