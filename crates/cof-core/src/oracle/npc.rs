use std::fmt;

use super::data::TABLES;
use crate::dice::Die;

/// What the player knows about an NPC before meeting them, which modifies their Initial
/// Reaction roll (p.90). Monsters and mindless beasts don't roll: they always attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Disposition {
    #[default]
    Unknown,
    /// A merchant, or someone who wants something: +2.
    WantsSomething,
    /// Avoiding people or hiding something: -2.
    Hiding,
    /// A loner: -4.
    Loner,
    /// Usually hostile (bandits, Penitents, Unmade…): -10.
    UsuallyHostile,
}

/// A randomly generated NPC (p.26–27, p.90–93).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Npc {
    pub name: &'static str,
    pub appearance: &'static str,
    pub age: &'static str,
    pub nature: &'static str,
    pub descriptor: &'static str,
    pub emotion: &'static str,
    pub activity: &'static str,
    pub motivation: &'static str,
    pub disposition: Disposition,
    pub reaction_roll: u8,
    pub reaction: &'static str,
    pub reaction_detail: &'static str,
}

impl Disposition {
    pub const ALL: [Disposition; 5] = [
        Disposition::Unknown,
        Disposition::WantsSomething,
        Disposition::Hiding,
        Disposition::Loner,
        Disposition::UsuallyHostile,
    ];

    pub fn modifier(self) -> i8 {
        match self {
            Disposition::Unknown => 0,
            Disposition::WantsSomething => 2,
            Disposition::Hiding => -2,
            Disposition::Loner => -4,
            Disposition::UsuallyHostile => -10,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Disposition::Unknown => "No modifier",
            Disposition::WantsSomething => "Wants something",
            Disposition::Hiding => "Hiding something",
            Disposition::Loner => "Loner",
            Disposition::UsuallyHostile => "Usually hostile",
        }
    }

    /// The next disposition in [`Disposition::ALL`], wrapping around.
    pub fn cycled(self, steps: i8) -> Disposition {
        let len = Self::ALL.len() as i16;
        let index = (self as i16 + i16::from(steps)).rem_euclid(len);
        Self::ALL[index as usize]
    }

    /// The reaction for a D10 roll: modifier applied, clamped to the table (1–10).
    pub fn reaction_total(self, roll: u8) -> u8 {
        (roll as i8 + self.modifier()).clamp(1, 10) as u8
    }
}

/// Rolls up an NPC: name, look, personality, what they're doing, and how they react.
pub fn generate_npc(disposition: Disposition) -> Npc {
    let t = &*TABLES;
    let appearance = t.genders.roll().value.as_str();
    let names = t.names.roll().value;
    let name = match appearance {
        "Female appearance" => &names.female,
        "Male appearance" => &names.male,
        // Androgynous: either column.
        _ if Die::D6.roll() <= 3 => &names.female,
        _ => &names.male,
    };
    let reaction_roll = Die::D10.roll();
    let reaction = t
        .reactions
        .lookup(disposition.reaction_total(reaction_roll))
        .expect("1–10 is covered");
    Npc {
        name,
        appearance,
        age: t.ages.roll().value,
        nature: t.natures.roll().value,
        descriptor: t.descriptors.roll().value,
        emotion: t.emotions.roll().value,
        activity: t.activities.roll().value,
        motivation: t.motivations.roll().value,
        disposition,
        reaction_roll,
        reaction,
        reaction_detail: t.reaction_details.roll().value,
    }
}

impl fmt::Display for Npc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {}, {}",
            self.name,
            self.appearance,
            self.age.to_lowercase()
        )?;
        writeln!(
            f,
            "{} and {}, feeling {}",
            self.nature,
            self.descriptor.to_lowercase(),
            self.emotion.to_lowercase()
        )?;
        writeln!(
            f,
            "Doing: {} · Motivation: {}",
            self.activity, self.motivation
        )?;
        write!(f, "Reaction: D10 = {}", self.reaction_roll)?;
        let modifier = self.disposition.modifier();
        if modifier != 0 {
            let total = self.disposition.reaction_total(self.reaction_roll);
            write!(f, " {modifier:+} = {total}")?;
        }
        write!(
            f,
            " → {} ({})",
            self.reaction,
            self.reaction_detail.to_lowercase()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_match_the_rulebook() {
        let mods: Vec<i8> = Disposition::ALL.iter().map(|d| d.modifier()).collect();
        assert_eq!(mods, [0, 2, -2, -4, -10]);
    }

    #[test]
    fn every_reaction_roll_clamps_into_the_table() {
        for disposition in Disposition::ALL {
            for roll in 1..=10 {
                let total = disposition.reaction_total(roll);
                assert!((1..=10).contains(&total));
                assert_eq!(
                    i16::from(total),
                    (i16::from(roll) + i16::from(disposition.modifier())).clamp(1, 10)
                );
            }
        }
        // Usually hostile is always Hostile.
        for roll in 1..=10 {
            assert_eq!(Disposition::UsuallyHostile.reaction_total(roll), 1);
        }
    }

    #[test]
    fn names_follow_appearance() {
        let names: Vec<_> = TABLES.names.entries().map(|(_, n)| n.clone()).collect();
        for _ in 0..2_000 {
            let npc = generate_npc(Disposition::Unknown);
            let is_female = names.iter().any(|n| n.female == npc.name);
            let is_male = names.iter().any(|n| n.male == npc.name);
            match npc.appearance {
                "Female appearance" => assert!(is_female, "{}", npc.name),
                "Male appearance" => assert!(is_male, "{}", npc.name),
                _ => assert!(is_female || is_male),
            }
        }
    }

    #[test]
    fn hostile_npcs_are_always_hostile() {
        for _ in 0..500 {
            assert_eq!(
                generate_npc(Disposition::UsuallyHostile).reaction,
                "Hostile"
            );
        }
    }

    #[test]
    fn cycling_wraps() {
        assert_eq!(Disposition::Unknown.cycled(-1), Disposition::UsuallyHostile);
        assert_eq!(Disposition::UsuallyHostile.cycled(1), Disposition::Unknown);
    }
}
