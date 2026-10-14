//! Gear & Spoils of War (Chapter VI): the catalog of things to buy, carry, wield and wear.

mod catalog;
mod types;

pub use catalog::{GAMBESON, RATIONS, catalog, find, proficiency_armor, weapons};
pub use types::{
    ArmorSlot, ArmorStats, DamageType, GearItem, GearKind, WeaponStats, WeaponTrait, Weight,
};
