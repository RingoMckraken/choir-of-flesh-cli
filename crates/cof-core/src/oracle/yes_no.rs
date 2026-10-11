use std::fmt;

use super::data::TABLES;
use crate::dice::Die;

/// How likely the answer to a yes/no question is (p.84).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Likelihood {
    AlmostImpossible,
    VeryUnlikely,
    Unlikely,
    #[default]
    FiftyFifty,
    Likely,
    VeryLikely,
    AlmostCertain,
}

/// The extra D6 turned up a 1 (p.84).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Twist {
    /// "Yes, but…": a potential complication.
    Complication,
    /// "No, but…": something positive might happen anyway.
    SilverLining,
}

/// The answer to a yes/no question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleAnswer {
    pub likelihood: Likelihood,
    pub roll: u8,
    pub yes: bool,
    pub twist_die: u8,
    pub twist: Option<Twist>,
    /// When there's a twist, a roll on the Complication table for ideas (p.85).
    pub idea: Option<(u8, &'static str)>,
}

impl Likelihood {
    pub const ALL: [Likelihood; 7] = [
        Likelihood::AlmostImpossible,
        Likelihood::VeryUnlikely,
        Likelihood::Unlikely,
        Likelihood::FiftyFifty,
        Likelihood::Likely,
        Likelihood::VeryLikely,
        Likelihood::AlmostCertain,
    ];

    /// The percentage chance of a "yes".
    pub fn chance(self) -> u8 {
        20 + 10 * self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Likelihood::AlmostImpossible => "Almost Impossible",
            Likelihood::VeryUnlikely => "Very Unlikely",
            Likelihood::Unlikely => "Unlikely",
            Likelihood::FiftyFifty => "Fifty-Fifty",
            Likelihood::Likely => "Likely",
            Likelihood::VeryLikely => "Very Likely",
            Likelihood::AlmostCertain => "Almost Certain",
        }
    }

    /// The next likelihood up or down, stopping at either end.
    pub fn shifted(self, steps: i8) -> Likelihood {
        let index = (self as i16 + i16::from(steps)).clamp(0, Self::ALL.len() as i16 - 1);
        Self::ALL[index as usize]
    }

    /// The answer for a D100 roll and a D6 roll the player made themselves.
    pub fn resolve(self, roll: u8, twist_die: u8) -> (bool, Option<Twist>) {
        let yes = roll <= self.chance();
        let twist = (twist_die == 1).then_some(if yes {
            Twist::Complication
        } else {
            Twist::SilverLining
        });
        (yes, twist)
    }
}

/// Asks the Yes/No Oracle (p.84): roll D100 against the likelihood, plus a D6 for a twist.
pub fn ask(likelihood: Likelihood) -> OracleAnswer {
    let roll = Die::D100.roll();
    let twist_die = Die::D6.roll();
    let (yes, twist) = likelihood.resolve(roll, twist_die);
    let idea = twist.map(|_| {
        let rolled = TABLES.complications.roll();
        (rolled.roll, rolled.value.as_str())
    });
    OracleAnswer {
        likelihood,
        roll,
        yes,
        twist_die,
        twist,
        idea,
    }
}

impl fmt::Display for OracleAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let answer = if self.yes { "YES" } else { "NO" };
        write!(
            f,
            "{} ({}%): D100 = {} → {answer}",
            self.likelihood.name(),
            self.likelihood.chance(),
            self.roll
        )?;
        match self.twist {
            None => write!(f, "\nD6 = {}: no twist.", self.twist_die),
            Some(twist) => {
                let label = match twist {
                    Twist::Complication => "Yes, but… a complication",
                    Twist::SilverLining => "No, but… something good might happen",
                };
                write!(f, "\nD6 = 1: {label}.")?;
                if let Some((roll, idea)) = self.idea {
                    write!(f, "\nIdea (D6 = {roll}): {idea}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chances_match_the_table() {
        let chances: Vec<u8> = Likelihood::ALL.iter().map(|l| l.chance()).collect();
        assert_eq!(chances, [20, 30, 40, 50, 60, 70, 80]);
    }

    #[test]
    fn every_d100_and_d6_result_resolves_by_the_rule() {
        for likelihood in Likelihood::ALL {
            for roll in 1..=100 {
                for twist_die in 1..=6 {
                    let (yes, twist) = likelihood.resolve(roll, twist_die);
                    assert_eq!(yes, roll <= likelihood.chance());
                    let expected = match (twist_die, yes) {
                        (1, true) => Some(Twist::Complication),
                        (1, false) => Some(Twist::SilverLining),
                        _ => None,
                    };
                    assert_eq!(twist, expected);
                }
            }
        }
    }

    #[test]
    fn real_asks_are_consistent_and_reach_every_outcome() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..20_000 {
            let answer = ask(Likelihood::FiftyFifty);
            assert_eq!(
                (answer.yes, answer.twist),
                Likelihood::FiftyFifty.resolve(answer.roll, answer.twist_die)
            );
            assert_eq!(answer.idea.is_some(), answer.twist.is_some());
            seen.insert((answer.yes, answer.twist));
        }
        assert_eq!(
            seen.len(),
            4,
            "yes, no, yes-but, and no-but should all come up"
        );
    }

    #[test]
    fn shifting_saturates() {
        assert_eq!(Likelihood::FiftyFifty.shifted(1), Likelihood::Likely);
        assert_eq!(
            Likelihood::AlmostCertain.shifted(5),
            Likelihood::AlmostCertain
        );
        assert_eq!(
            Likelihood::AlmostImpossible.shifted(-1),
            Likelihood::AlmostImpossible
        );
    }
}
