use serde::{Deserialize, Serialize};

/// How much carrying space an item takes (p.208).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weight {
    /// Takes no Item Slot: tiny things, empty bags, worn clothing and jewelry, or a "–" in
    /// the gear tables.
    NoSlot,
    /// Up to 10 bundle into one Item Slot (torches, rations, vials…).
    Light,
    /// One Item Slot.
    Normal,
    /// Two Item Slots.
    Heavy,
}

impl Weight {
    pub const ALL: [Weight; 4] = [Weight::NoSlot, Weight::Light, Weight::Normal, Weight::Heavy];

    /// How many Light items share one slot.
    pub const LIGHT_PER_SLOT: u16 = 10;

    pub fn name(self) -> &'static str {
        match self {
            Weight::NoSlot => "No slot",
            Weight::Light => "Light",
            Weight::Normal => "Normal",
            Weight::Heavy => "Heavy",
        }
    }

    /// Item Slots taken by `quantity` items of this weight. Light items bundle 10 to a slot,
    /// rounding up (each stack is bundled on its own).
    pub fn slots(self, quantity: u16) -> u16 {
        match self {
            Weight::NoSlot => 0,
            Weight::Light => quantity.div_ceil(Self::LIGHT_PER_SLOT),
            Weight::Normal => quantity,
            Weight::Heavy => quantity.saturating_mul(2),
        }
    }
}

/// Damage types (p.57–60). Each has its own Injury Effect table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Bludgeoning,
    Corrosive,
    Divine,
    Fire,
    Piercing,
    Primordial,
    Slashing,
}

impl DamageType {
    pub fn name(self) -> &'static str {
        match self {
            DamageType::Bludgeoning => "Bludgeoning",
            DamageType::Corrosive => "Corrosive",
            DamageType::Divine => "Divine",
            DamageType::Fire => "Fire",
            DamageType::Piercing => "Piercing",
            DamageType::Primordial => "Primordial",
            DamageType::Slashing => "Slashing",
        }
    }
}

/// Weapon Traits (p.211).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponTrait {
    Agile,
    Brutal,
    Concussive,
    Finesse,
    Guard,
    OneHanded,
    /// Maximum range in squares.
    Ranged(u8),
    Reach,
    /// -2 Initiative.
    Slow,
    Sweep,
    TwoHanded,
    Versatile,
    Vicious,
}

impl WeaponTrait {
    pub fn label(self) -> String {
        match self {
            WeaponTrait::Agile => "Agile".into(),
            WeaponTrait::Brutal => "Brutal".into(),
            WeaponTrait::Concussive => "Concussive".into(),
            WeaponTrait::Finesse => "Finesse".into(),
            WeaponTrait::Guard => "Guard".into(),
            WeaponTrait::OneHanded => "One-Handed".into(),
            WeaponTrait::Ranged(range) => format!("Ranged ({range})"),
            WeaponTrait::Reach => "Reach".into(),
            WeaponTrait::Slow => "Slow".into(),
            WeaponTrait::Sweep => "Sweep".into(),
            WeaponTrait::TwoHanded => "Two-Handed".into(),
            WeaponTrait::Versatile => "Versatile".into(),
            WeaponTrait::Vicious => "Vicious".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponStats {
    pub damage: Vec<DamageType>,
    pub traits: Vec<WeaponTrait>,
}

/// Where a piece of armor is worn. A character wears at most one of each (the Gambeson is
/// the padding layer under body armor, p.215).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorSlot {
    Padding,
    Body,
    Helmet,
    Shield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorStats {
    pub slot: ArmorSlot,
    /// Added to Defense Rating.
    #[serde(default)]
    pub dr: u8,
    /// Shields: Mastery on DR checks rather than DR (p.216).
    #[serde(default)]
    pub defense_mastery: u8,
    #[serde(default)]
    pub min_str: Option<u8>,
    #[serde(default)]
    pub stealth_disadvantage: bool,
}

impl ArmorStats {
    /// Helmets don't need a Proficiency; everything else does (p.14).
    pub fn needs_proficiency(&self) -> bool {
        self.slot != ArmorSlot::Helmet
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GearKind {
    Weapon(WeaponStats),
    Armor(ArmorStats),
    /// +10 Item Slots each, up to two (p.208).
    Backpack,
    #[default]
    Gear,
}

/// One entry of the gear catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GearItem {
    pub name: String,
    /// In Rations.
    pub cost: u16,
    pub weight: Weight,
    #[serde(default)]
    pub kind: GearKind,
    #[serde(default)]
    pub notes: String,
}

impl GearItem {
    pub fn weapon(&self) -> Option<&WeaponStats> {
        match &self.kind {
            GearKind::Weapon(w) => Some(w),
            _ => None,
        }
    }

    pub fn armor(&self) -> Option<&ArmorStats> {
        match &self.kind {
            GearKind::Armor(a) => Some(a),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_per_weight() {
        for q in 0..=40 {
            assert_eq!(Weight::NoSlot.slots(q), 0);
            assert_eq!(Weight::Normal.slots(q), q);
            assert_eq!(Weight::Heavy.slots(q), 2 * q);
        }
        assert_eq!(Weight::Light.slots(0), 0);
        assert_eq!(Weight::Light.slots(1), 1);
        assert_eq!(Weight::Light.slots(10), 1);
        assert_eq!(Weight::Light.slots(11), 2);
        assert_eq!(Weight::Light.slots(30), 3);
    }
}
