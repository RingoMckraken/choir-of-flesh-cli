use crate::dice::Die;

/// Breakdown result is a status that happens when accumulating a certain threshold of
/// anguish. The character will have to roll on a table and then accept the Breakdown result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakdownResult {
    PanicAttack,
    ViolentOutburst,
    DesperateFlight,
    CompulsiveLie,
    Fixation,
    NewPhobia,
    SinnersResolve,
    Catatonia,
    TheVoiceWithin,
    MomentOfClarity,
}

pub struct Anguish {
    current: u8,
}

impl Default for Anguish {
    fn default() -> Self {
        Self::new()
    }
}

impl Anguish {
    pub fn new() -> Self {
        Self { current: 0 }
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn gain(&mut self, amount: u8) {
        self.current = self.current.saturating_add(amount);
        // self.breakdown_check() todo
    }

    pub fn lose(&mut self, amount: u8) {
        self.current = self.current.saturating_sub(amount);
    }

    fn breakdown_check(&self) -> Option<BreakdownResult> {
        let roll = Die::D20.roll();
        if roll <= self.current {
            Some(Self::roll_breakdown_table())
        } else {
            None
        }
    }

    fn roll_breakdown_table() -> BreakdownResult {
        match Die::D10.roll() {
            1 => BreakdownResult::PanicAttack,
            2 => BreakdownResult::ViolentOutburst,
            3 => BreakdownResult::DesperateFlight,
            4 => BreakdownResult::CompulsiveLie,
            5 => BreakdownResult::Fixation,
            6 => BreakdownResult::NewPhobia,
            7 => BreakdownResult::SinnersResolve,
            8 => BreakdownResult::Catatonia,
            9 => BreakdownResult::TheVoiceWithin,
            _ => BreakdownResult::MomentOfClarity,
        }
    }
}
