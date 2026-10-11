/// Conditions (p.62–63). Variants carrying a number are the "(X)" Conditions.
///
/// External durations ("Hindered for D6 rounds", "Sickened (2) for D4 rounds") are tracked
/// by whatever applied the Condition (combat, in Milestone 6). Only the counters built into a
/// Condition itself are handled here, by [`Condition::tick`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Condition {
    /// Free Average CON check each round: fail = 1 Minor Injury, pass = removed.
    Bleeding,
    /// -2 Burden on all checks; can only target up to two squares away. Lasts D4 rounds.
    Blinded,
    /// 1 Minor Injury per round until a Standard Action puts it out.
    Burning,
    /// Can't attack or harm the charmer. Average WIL check each round to shake it off.
    Charmed,
    /// Can't initiate attacks for X rounds.
    Dazed(u8),
    /// Disadvantage on INT, WIL, and PRE checks. Removed by 6 hours of sleep.
    Fatigued,
    /// Burden X on actions against the source of fear. X drops by 1 at the end of each turn.
    Frightened(u8),
    /// Move reduced by 1.
    Hindered,
    /// Free Average CON check each round: fail = 1 Minor Injury, pass = removed.
    Poisoned,
    /// Attacks against the creature have Advantage. Costs all Move to stand up.
    Prone,
    /// Can't move; actions that involve movement (like attacking) have Disadvantage.
    Restrained,
    /// Burden X on everything.
    Sickened(u8),
    /// Disadvantage on all checks. Removed by eating a Ration.
    Starved,
    /// No Standard Actions for X rounds (can still move and take Free Actions).
    Stunned(u8),
    /// Loses 1 CON per round until able to breathe again.
    Suffocating,
}

impl Condition {
    pub fn name(self) -> &'static str {
        match self {
            Condition::Bleeding => "Bleeding",
            Condition::Blinded => "Blinded",
            Condition::Burning => "Burning",
            Condition::Charmed => "Charmed",
            Condition::Dazed(_) => "Dazed",
            Condition::Fatigued => "Fatigued",
            Condition::Frightened(_) => "Frightened",
            Condition::Hindered => "Hindered",
            Condition::Poisoned => "Poisoned",
            Condition::Prone => "Prone",
            Condition::Restrained => "Restrained",
            Condition::Sickened(_) => "Sickened",
            Condition::Starved => "Starved",
            Condition::Stunned(_) => "Stunned",
            Condition::Suffocating => "Suffocating",
        }
    }

    /// The X of an "(X)" Condition.
    pub fn value(self) -> Option<u8> {
        match self {
            Condition::Dazed(x)
            | Condition::Frightened(x)
            | Condition::Sickened(x)
            | Condition::Stunned(x) => Some(x),
            _ => None,
        }
    }

    /// Burden this Condition puts on every check the creature makes. Frightened only
    /// applies against its source, so it isn't included.
    pub fn general_burden(self) -> u8 {
        match self {
            Condition::Sickened(x) => x,
            Condition::Blinded => 2,
            _ => 0,
        }
    }

    /// Advances a self-counting Condition by one turn. Dazed and Stunned count rounds down,
    /// and Frightened drops by 1 at the end of each of the afflicted creature's turns.
    ///
    /// Returns `None` when the Condition wears off. Every other Condition is returned
    /// unchanged.
    pub fn tick(self) -> Option<Condition> {
        let countdown = |x: u8, make: fn(u8) -> Condition| (x > 1).then(|| make(x - 1));
        match self {
            Condition::Dazed(x) => countdown(x, Condition::Dazed),
            Condition::Stunned(x) => countdown(x, Condition::Stunned),
            Condition::Frightened(x) => countdown(x, Condition::Frightened),
            other => Some(other),
        }
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value() {
            Some(x) => write!(f, "{} ({x})", self.name()),
            None => f.write_str(self.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_tick_down_and_wear_off() {
        assert_eq!(Condition::Stunned(2).tick(), Some(Condition::Stunned(1)));
        assert_eq!(Condition::Stunned(1).tick(), None);
        assert_eq!(Condition::Dazed(1).tick(), None);
        assert_eq!(
            Condition::Frightened(3).tick(),
            Some(Condition::Frightened(2))
        );
    }

    #[test]
    fn other_conditions_do_not_tick() {
        assert_eq!(Condition::Sickened(2).tick(), Some(Condition::Sickened(2)));
        assert_eq!(Condition::Bleeding.tick(), Some(Condition::Bleeding));
    }

    #[test]
    fn general_burden() {
        assert_eq!(Condition::Sickened(2).general_burden(), 2);
        assert_eq!(Condition::Blinded.general_burden(), 2);
        assert_eq!(Condition::Frightened(2).general_burden(), 0);
        assert_eq!(Condition::Prone.general_burden(), 0);
    }

    #[test]
    fn display_includes_value() {
        assert_eq!(Condition::Sickened(2).to_string(), "Sickened (2)");
        assert_eq!(Condition::Prone.to_string(), "Prone");
    }
}
