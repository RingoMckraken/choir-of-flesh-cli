use serde::{Deserialize, Serialize};

use crate::gear::{self, ArmorSlot, GearItem, GearKind, WeaponTrait, Weight};

/// The most Backpacks a character can equip (p.208).
pub const MAX_BACKPACKS: usize = 2;
/// Item Slots per equipped Backpack.
pub const BACKPACK_SLOTS: u16 = 10;

/// Something carried. Items from the gear catalog keep their catalog name, so their
/// mechanics (DR, traits, weight) are looked up rather than copied. Anything else (loot,
/// relics, apocalyptic items) is a custom item with a weight the player picks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub weight: Weight,
    pub quantity: u16,
    /// Worn or wielded: armor counts toward DR, a Backpack adds slots, a Slow weapon
    /// lowers Initiative. Equipped things still take Item Slots (p.208).
    #[serde(default)]
    pub equipped: bool,
    /// +1 Mastery to attacks (weapons) or defensive checks (armor), p.219.
    #[serde(default)]
    pub masterwork: bool,
    #[serde(default)]
    pub notes: String,
}

impl Item {
    pub fn from_catalog(gear: &GearItem, quantity: u16) -> Self {
        Self {
            name: gear.name.clone(),
            weight: gear.weight,
            quantity,
            equipped: false,
            masterwork: false,
            notes: String::new(),
        }
    }

    pub fn custom(name: impl Into<String>, weight: Weight, quantity: u16) -> Self {
        Self {
            name: name.into(),
            weight,
            quantity,
            equipped: false,
            masterwork: false,
            notes: String::new(),
        }
    }

    /// The catalog entry, if this is book gear.
    pub fn gear(&self) -> Option<&'static GearItem> {
        gear::find(&self.name)
    }

    pub fn slots(&self) -> u16 {
        self.weight.slots(self.quantity)
    }

    fn armor_slot(&self) -> Option<ArmorSlot> {
        self.gear().and_then(|g| g.armor()).map(|a| a.slot)
    }

    fn is_backpack(&self) -> bool {
        self.gear()
            .is_some_and(|g| matches!(g.kind, GearKind::Backpack))
    }
}

/// Everything a character carries, plus their Rations (Light, so 10 per slot, p.208; the
/// sheet's Rations box is just a convenient view of them).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Item>,
    #[serde(default)]
    pub rations: u16,
}

impl Inventory {
    /// Item Slots in use, Rations included.
    pub fn slots_used(&self) -> u16 {
        self.items.iter().map(Item::slots).sum::<u16>() + Weight::Light.slots(self.rations)
    }

    /// Equipped Backpacks (at most two count).
    pub fn backpacks(&self) -> u16 {
        let n = self
            .items
            .iter()
            .filter(|i| i.equipped && i.is_backpack())
            .map(|i| usize::from(i.quantity))
            .sum::<usize>();
        n.min(MAX_BACKPACKS) as u16
    }

    /// DR from equipped armor and helmets (shields give Mastery instead).
    pub fn armor_dr(&self) -> u8 {
        self.equipped_gear()
            .filter_map(|g| g.armor())
            .map(|a| a.dr)
            .sum()
    }

    /// Mastery on DR checks from an equipped shield, plus +1 for each equipped Masterwork
    /// armor piece (p.216, p.219).
    pub fn defense_mastery(&self) -> u8 {
        let shields: u8 = self
            .equipped_gear()
            .filter_map(|g| g.armor())
            .map(|a| a.defense_mastery)
            .sum();
        let masterwork = self
            .items
            .iter()
            .filter(|i| i.equipped && i.masterwork && i.armor_slot().is_some())
            .count() as u8;
        shields + masterwork
    }

    /// True while wielding a weapon with the Slow trait (-2 Initiative, p.211).
    pub fn wielding_slow_weapon(&self) -> bool {
        self.equipped_gear()
            .filter_map(|g| g.weapon())
            .any(|w| w.traits.contains(&WeaponTrait::Slow))
    }

    /// The equipped catalog items.
    pub fn equipped_gear(&self) -> impl Iterator<Item = &'static GearItem> + '_ {
        self.items
            .iter()
            .filter(|i| i.equipped)
            .filter_map(Item::gear)
    }

    /// How many of an item by name (e.g. "Medical Supplies").
    pub fn count(&self, name: &str) -> u16 {
        self.items
            .iter()
            .filter(|i| i.name == name)
            .map(|i| i.quantity)
            .sum()
    }

    /// Adds an item, stacking it onto an identical unequipped stack.
    pub fn add(&mut self, item: Item) {
        let same = |i: &&mut Item| {
            !i.equipped
                && i.name == item.name
                && i.weight == item.weight
                && i.masterwork == item.masterwork
                && i.notes == item.notes
        };
        match self.items.iter_mut().find(same) {
            Some(stack) if !item.equipped => {
                stack.quantity = stack.quantity.saturating_add(item.quantity);
            }
            _ => self.items.push(item),
        }
    }

    /// Removes up to `quantity` from the stack at `index`, dropping it when empty. Returns
    /// what was removed.
    pub fn take(&mut self, index: usize, quantity: u16) -> Option<Item> {
        let stack = self.items.get_mut(index)?;
        let taken = quantity.min(stack.quantity);
        let mut removed = stack.clone();
        removed.quantity = taken;
        removed.equipped = false;
        stack.quantity -= taken;
        if stack.quantity == 0 {
            self.items.remove(index);
        }
        Some(removed)
    }

    /// Equips or unequips the item at `index`. Equipping armor takes off whatever is in the
    /// same armor slot. A third Backpack can't be equipped. Returns false if nothing changed.
    pub fn toggle_equipped(&mut self, index: usize) -> bool {
        let Some(item) = self.items.get(index) else {
            return false;
        };
        if item.equipped {
            self.items[index].equipped = false;
            return true;
        }
        if item.is_backpack() && self.backpacks() as usize >= MAX_BACKPACKS {
            return false;
        }
        if let Some(slot) = item.armor_slot() {
            for (i, other) in self.items.iter_mut().enumerate() {
                if i != index && other.equipped && other.armor_slot() == Some(slot) {
                    other.equipped = false;
                }
            }
        }
        self.items[index].equipped = true;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str) -> Item {
        Item::from_catalog(gear::find(name).unwrap(), 1)
    }

    #[test]
    fn slots_count_every_weight_and_the_rations() {
        let mut inv = Inventory {
            rations: 25,
            ..Inventory::default()
        };
        inv.add(item("Greatsword")); // Heavy: 2
        inv.add(item("Rope")); // Normal: 1
        inv.add(Item::from_catalog(gear::find("Torch").unwrap(), 12)); // Light: 2
        inv.add(item("Backpack")); // no slot
        assert_eq!(inv.slots_used(), 2 + 1 + 2 + 3);
    }

    #[test]
    fn armor_slots_are_exclusive_and_dr_adds_up() {
        let mut inv = Inventory::default();
        for name in [
            "Gambeson",
            "Chainmail",
            "Scale Armor",
            "Leather Cap",
            "Light Shield",
        ] {
            inv.add(item(name));
        }
        for i in 0..inv.items.len() {
            assert!(inv.toggle_equipped(i));
        }
        // Scale Armor replaced Chainmail in the Body slot.
        assert!(!inv.items[1].equipped);
        assert_eq!(inv.armor_dr(), 1 + 3 + 1);
        assert_eq!(inv.defense_mastery(), 1);
        inv.items[2].masterwork = true;
        assert_eq!(inv.defense_mastery(), 2);
    }

    #[test]
    fn at_most_two_backpacks() {
        let mut inv = Inventory::default();
        for _ in 0..3 {
            inv.items.push(item("Backpack"));
        }
        assert!(inv.toggle_equipped(0) && inv.toggle_equipped(1));
        assert!(!inv.toggle_equipped(2));
        assert_eq!(inv.backpacks(), 2);
    }

    #[test]
    fn stacking_and_taking() {
        let mut inv = Inventory::default();
        inv.add(item("Torch"));
        inv.add(Item::from_catalog(gear::find("Torch").unwrap(), 4));
        assert_eq!((inv.items.len(), inv.count("Torch")), (1, 5));
        let taken = inv.take(0, 2).unwrap();
        assert_eq!((taken.quantity, inv.count("Torch")), (2, 3));
        inv.take(0, 10);
        assert!(inv.items.is_empty());
    }

    #[test]
    fn a_slow_weapon_only_counts_when_wielded() {
        let mut inv = Inventory::default();
        inv.add(item("Maul"));
        assert!(!inv.wielding_slow_weapon());
        inv.toggle_equipped(0);
        assert!(inv.wielding_slow_weapon());
    }
}
