use crate::dice::Die;

/// The Breakdown table (D10, p.49): what happens when a character fails an Anguish Check.
///
/// Resolving a Breakdown is left to the player and the caller, because several involve
/// choices (Moment of Clarity) or story. The two with a direct effect on Anguish have
/// helpers on [`Anguish`](super::Anguish): `gain_permanent` for The Voice Within and
/// `defy_clarity` for Moment of Clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Breakdown {
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

impl Breakdown {
    /// Table order: index 0 is a roll of 1.
    pub const ALL: [Breakdown; 10] = [
        Breakdown::PanicAttack,
        Breakdown::ViolentOutburst,
        Breakdown::DesperateFlight,
        Breakdown::CompulsiveLie,
        Breakdown::Fixation,
        Breakdown::NewPhobia,
        Breakdown::SinnersResolve,
        Breakdown::Catatonia,
        Breakdown::TheVoiceWithin,
        Breakdown::MomentOfClarity,
    ];

    /// Looks up a D10 result (1–10). Returns `None` for anything else.
    pub fn from_roll(roll: u8) -> Option<Breakdown> {
        Self::ALL.get(usize::from(roll).checked_sub(1)?).copied()
    }

    /// Rolls a D10 on the table.
    pub fn roll() -> Breakdown {
        Self::ALL[usize::from(Die::D10.roll()) - 1]
    }

    pub fn name(self) -> &'static str {
        match self {
            Breakdown::PanicAttack => "Panic Attack",
            Breakdown::ViolentOutburst => "Violent Outburst",
            Breakdown::DesperateFlight => "Desperate Flight",
            Breakdown::CompulsiveLie => "Compulsive Lie",
            Breakdown::Fixation => "Fixation",
            Breakdown::NewPhobia => "New Phobia",
            Breakdown::SinnersResolve => "Sinner's Resolve",
            Breakdown::Catatonia => "Catatonia",
            Breakdown::TheVoiceWithin => "The Voice Within",
            Breakdown::MomentOfClarity => "Moment of Clarity",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn table_order_matches_rulebook() {
        let names: Vec<_> = (1..=10)
            .map(|roll| Breakdown::from_roll(roll).unwrap().name())
            .collect();
        assert_eq!(
            names,
            [
                "Panic Attack",
                "Violent Outburst",
                "Desperate Flight",
                "Compulsive Lie",
                "Fixation",
                "New Phobia",
                "Sinner's Resolve",
                "Catatonia",
                "The Voice Within",
                "Moment of Clarity",
            ]
        );
    }

    #[test]
    fn out_of_range_rolls_are_rejected() {
        assert_eq!(Breakdown::from_roll(0), None);
        assert_eq!(Breakdown::from_roll(11), None);
    }

    #[test]
    fn rolling_reaches_every_entry() {
        let seen: HashSet<_> = (0..5_000).map(|_| Breakdown::roll()).collect();
        assert_eq!(seen.len(), Breakdown::ALL.len());
    }
}
