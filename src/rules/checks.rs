use crate::dice::Die;
/// Difficultly is used to define how difficult a check will be as
/// well as how difficult an enemy is and some other to be defined
/// later
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    Average,
    Hard,
    VeryHard,
    NearlyImpossible,
}

impl Difficulty {
    /// Base Target number offset with no modifiers
    pub fn tn_offset(&self) -> u8 {
        match self {
            Difficulty::VeryEasy => 7,
            Difficulty::Easy => 9,
            Difficulty::Average => 11,
            Difficulty::Hard => 13,
            Difficulty::VeryHard => 15,
            Difficulty::NearlyImpossible => 17,
        }
    }
}

/// helper method to fetch a dynamic tn based on a modifier
pub fn dynamic_tn(difficulty: Difficulty, modifier: i8) -> u8 {
    let tn = modifier + difficulty.tn_offset() as i8;
    tn.clamp(2, 27) as u8
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CheckResult {
    CriticalSuccess,
    Success,
    Failure,
    CriticalFailure
}

#[derive(Debug)]
pub struct CheckOutcome {
    pub result: CheckResult,
    pub roll: u8,
    pub margin: i8,
}

/// roll_check
pub fn roll_check(modifier: i8, tn: u8) -> CheckOutcome {
    let roll = Die::D20.roll();

    // Critical Success
    if roll == 20 {
        return CheckOutcome {
            result: CheckResult::CriticalSuccess,
            roll,
            margin: (roll as i8 + modifier) - tn as i8,
        };
    }

    if roll == 1 {
        return CheckOutcome {
            result: CheckResult::CriticalFailure,
            roll,
            margin: (roll as i8 + modifier) - tn as i8,
        };
    }

    let total = roll as i8 + modifier;
    let margin = total - tn as i8;
    let result = if margin >= 0 {
        CheckResult::Success
    } else {
        CheckResult::Failure
    };

    CheckOutcome {
        result,
        roll,
        margin,
    }
}

pub fn roll_dynamic_check(
    difficulty: Difficulty,
    modifier: i8,
    mastery: i8,
    burden: i8,
) -> CheckOutcome {
    let tn = dynamic_tn(difficulty, modifier);
    let roll = Die::D20.roll();

    if roll == 20 {
        return CheckOutcome {
            result: CheckResult::CriticalSuccess,
            roll,
            margin: (roll as i8 + modifier + mastery + burden) - tn as i8,
        };
    }

    if roll == 1 {
        return CheckOutcome {
            result: CheckResult::CriticalFailure,
            roll,
            margin: (roll as i8 + modifier + mastery + burden) - tn as i8
        };
    }

    let total = roll as i8 + modifier + mastery + burden;
    let margin = total - tn as i8;

    let result = if margin >= 0 {
        CheckResult::Success
    } else {
        CheckResult::Failure
    };

    CheckOutcome{
        result,
        roll,
        margin,
    }
}