use serde::{Deserialize, Serialize};

use super::attributes::Attribute;
use super::data::DESCRIPTIONS;

macro_rules! skills {
    ($( $attr:ident => [ $( $variant:ident = $name:literal ),+ $(,)? ] ),+ $(,)?) => {
        /// The Skills (p.16–18), ten per Attribute. Descriptions are in
        /// `data/character/skills.ron`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum Skill {
            $( $( $variant, )+ )+
        }

        impl Skill {
            /// Every Skill, in book order (STR's ten, then CON's, and so on).
            pub const ALL: [Skill; 60] = [ $( $( Skill::$variant, )+ )+ ];

            pub fn name(self) -> &'static str {
                match self {
                    $( $( Skill::$variant => $name, )+ )+
                }
            }

            /// The Attribute whose list the Skill is on.
            pub fn attribute(self) -> Attribute {
                match self {
                    $( $( Skill::$variant => Attribute::$attr, )+ )+
                }
            }
        }
    };
}

skills! {
    Str => [
        Athlete = "Athlete",
        BattleFrenzy = "Battle Frenzy",
        Brace = "Brace",
        BruteForce = "Brute Force",
        GrapplingMastery = "Grappling Mastery",
        HeavyLifting = "Heavy Lifting",
        Intimidation = "Intimidation",
        MightyLeap = "Mighty Leap",
        PowerStrikes = "Power Strikes",
        TwoHandedWeaponMastery = "Two-Handed Weapon Mastery",
    ],
    Con => [
        EndlessStamina = "Endless Stamina",
        Fortitude = "Fortitude",
        Hardy = "Hardy",
        IronStomach = "Iron Stomach",
        PainTolerance = "Pain Tolerance",
        PoisonResistance = "Poison Resistance",
        QuickRecovery = "Quick Recovery",
        Tough = "Tough",
        Unbreakable = "Unbreakable",
        Vigorous = "Vigorous",
    ],
    Dex => [
        Acrobatics = "Acrobatics",
        Ambusher = "Ambusher",
        Evasion = "Evasion",
        Lockpicking = "Lockpicking",
        NimbleFingers = "Nimble Fingers",
        PrecisionStrikes = "Precision Strikes",
        QuickDraw = "Quick Draw",
        Stealth = "Stealth",
        TwoHandMastery = "Two Hand Mastery",
        VaultAndShoot = "Vault & Shoot",
    ],
    Int => [
        Healing = "Healing",
        Investigation = "Investigation",
        LipReading = "Lip Reading",
        LoreMastery = "Lore Mastery",
        Naturalism = "Naturalism",
        Navigation = "Navigation",
        Perception = "Perception",
        PuzzleSolving = "Puzzle Solving",
        Tactics = "Tactics",
        Theology = "Theology",
    ],
    Wil => [
        Calm = "Calm",
        Empathic = "Empathic",
        Insight = "Insight",
        InnerStrength = "Inner Strength",
        MentalFortitude = "Mental Fortitude",
        MindOverMatter = "Mind Over Matter",
        PsychicResistance = "Psychic Resistance",
        SoulResistance = "Soul Resistance",
        SelfReflection = "Self-Reflection",
        SuppressTheChange = "Suppress The Change",
    ],
    Pre => [
        Awe = "Awe",
        Bluff = "Bluff",
        Charm = "Charm",
        Command = "Command",
        Diplomacy = "Diplomacy",
        Haggling = "Haggling",
        Intimidating = "Intimidating",
        Leadership = "Leadership",
        Networking = "Networking",
        Performance = "Performance",
    ],
}

impl Skill {
    pub fn description(self) -> &'static str {
        &DESCRIPTIONS.skills[&self]
    }

    /// Only Suppress The Change can be taken more than once (p.18).
    pub fn repeatable(self) -> bool {
        self == Skill::SuppressTheChange
    }

    /// The Skills on one Attribute's list, e.g. for the bonus Skill at 16+.
    pub fn for_attribute(attribute: Attribute) -> impl Iterator<Item = Skill> {
        Skill::ALL
            .into_iter()
            .filter(move |s| s.attribute() == attribute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_skills_per_attribute() {
        for attribute in Attribute::ALL {
            assert_eq!(Skill::for_attribute(attribute).count(), 10, "{attribute:?}");
        }
    }

    #[test]
    fn every_skill_has_a_description() {
        for skill in Skill::ALL {
            assert!(!skill.description().is_empty(), "{skill:?}");
        }
        assert_eq!(DESCRIPTIONS.skills.len(), 60);
    }

    #[test]
    fn spot_checks_against_the_rulebook() {
        assert_eq!(Skill::Hardy.attribute(), Attribute::Con);
        assert_eq!(Skill::Theology.attribute(), Attribute::Int);
        // The Noble occupation mislabels these as WIL; they're on the PRE list (p.18).
        assert_eq!(Skill::Diplomacy.attribute(), Attribute::Pre);
        assert_eq!(Skill::Leadership.attribute(), Attribute::Pre);
        assert_eq!(
            Skill::Empathic.description(),
            "Your max Humanity is increased by 2."
        );
        assert!(Skill::SuppressTheChange.repeatable() && !Skill::Hardy.repeatable());
    }
}
