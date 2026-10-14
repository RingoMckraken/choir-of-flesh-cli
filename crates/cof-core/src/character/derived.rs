//! Derived stats (p.14, p.224). Computed from the character every time, never stored.

use super::background::gifts;
use super::inventory::BACKPACK_SLOTS;
use super::{Attribute, Character, Drawback, Feat, Skill};

/// Every PC's base Move (p.14).
pub const BASE_MOVE: u8 = 2;
/// A PC's Injury Track (p.14, p.56).
pub const MAX_MAJOR_INJURIES: u8 = 4;
pub const MAX_MINOR_INJURIES: u8 = 8;
/// With this many Gifts and Stigmata combined, God no longer hears your Prayers (p.71).
pub const TOO_FAR_GONE: usize = 5;

impl Character {
    pub fn modifier(&self, attribute: Attribute) -> i8 {
        self.attributes.modifier(attribute)
    }

    fn has_gift(&self, gift: u8) -> bool {
        self.gifts.contains(&gift)
    }

    /// DEX modifier + armor and helmet, + Unbreakable, + Chitinous Plates (p.14, p.17, p.70).
    pub fn defense_rating(&self) -> i8 {
        self.modifier(Attribute::Dex)
            + self.inventory.armor_dr() as i8
            + i8::from(self.has_skill(Skill::Unbreakable))
            + i8::from(self.has_gift(gifts::CHITINOUS_PLATES))
    }

    /// Mastery on DR checks from a shield and Masterwork armor (p.216, p.219).
    pub fn defense_mastery(&self) -> u8 {
        self.inventory.defense_mastery()
    }

    /// DEX modifier, +2 Quick Draw, -2 Slow drawback, -2 Broken Trauma, -2 wielding a Slow
    /// weapon (p.14, p.15, p.17, p.61, p.211).
    pub fn initiative(&self) -> i8 {
        let mut total = self.modifier(Attribute::Dex) + self.trauma.initiative_modifier();
        if self.has_skill(Skill::QuickDraw) {
            total += 2;
        }
        if self.has_drawback(Drawback::Slow) {
            total -= 2;
        }
        if self.inventory.wielding_slow_weapon() {
            total -= 2;
        }
        total
    }

    /// CON modifier + WIL modifier, +1 Vigorous (p.14, p.17). It never goes below 0.
    pub fn max_vigor(&self) -> u8 {
        let total = self.modifier(Attribute::Con)
            + self.modifier(Attribute::Wil)
            + i8::from(self.has_skill(Skill::Vigorous));
        total.max(0) as u8
    }

    /// Vigor left until the next 6-hour rest.
    pub fn vigor(&self) -> u8 {
        self.max_vigor().saturating_sub(self.vigor_spent)
    }

    /// 2, +1 Skirmisher, +1 Slithering Tail (p.14, p.15, p.71). Jumping is half of this.
    pub fn move_speed(&self) -> u8 {
        BASE_MOVE
            + u8::from(self.has_feat(Feat::Skirmisher))
            + u8::from(self.has_gift(gifts::SLITHERING_TAIL))
    }

    /// STR, +5 Heavy Lifting, halved while Worn by Trauma (p.14, p.16, p.61).
    pub fn carrying_capacity(&self) -> u8 {
        let base = self.attributes.score(Attribute::Str)
            + if self.has_skill(Skill::HeavyLifting) {
                5
            } else {
                0
            };
        self.trauma.carrying_capacity(base)
    }

    /// Carrying Capacity, +10 per equipped Backpack (up to two), +5 Corpulent Storage
    /// (p.208, p.71).
    pub fn item_slots(&self) -> u16 {
        u16::from(self.carrying_capacity())
            + self.inventory.backpacks() * BACKPACK_SLOTS
            + if self.has_gift(gifts::CORPULENT_STORAGE) {
                5
            } else {
                0
            }
    }

    /// Carrying more than you can means Disadvantage on all physical rolls (p.208).
    pub fn is_encumbered(&self) -> bool {
        self.inventory.slots_used() > self.item_slots()
    }

    /// 4, or 3 once Shattered by Trauma (p.61).
    pub fn max_major_injuries(&self) -> u8 {
        MAX_MAJOR_INJURIES - self.trauma.major_injury_track_reduction()
    }

    /// Modifier to Anguish Check rolls: +2 Calm, -1 Shaken by Trauma (p.18, p.61).
    pub fn anguish_check_modifier(&self) -> i8 {
        2 * i8::from(self.has_skill(Skill::Calm)) + self.trauma.anguish_check_modifier()
    }

    /// Extra Anguish whenever any is gained: +1 Cowardly, +1 Broken by Trauma (p.15, p.61).
    pub fn extra_anguish(&self) -> u8 {
        u8::from(self.has_drawback(Drawback::Cowardly)) + self.trauma.extra_anguish()
    }

    /// Five or more Gifts and Stigmata: no more Prayers (p.71).
    pub fn too_far_gone(&self) -> bool {
        self.gifts.len() + self.stigmata.len() >= TOO_FAR_GONE
    }
}

#[cfg(test)]
mod tests {
    use crate::character::unbroken::fixture::character;
    use crate::character::{Attribute, Drawback, Feat, Item, Skill};
    use crate::gear;

    fn equip(c: &mut crate::character::Character, name: &str) {
        c.inventory
            .add(Item::from_catalog(gear::find(name).unwrap(), 1));
        let i = c.inventory.items.len() - 1;
        assert!(c.inventory.toggle_equipped(i));
    }

    #[test]
    fn defense_rating_stacks_its_sources() {
        let mut c = character(); // DEX 14: +1
        assert_eq!(c.defense_rating(), 1);
        equip(&mut c, "Gambeson");
        equip(&mut c, "Spangenhelm");
        assert_eq!(c.defense_rating(), 1 + 1 + 2);
        c.learn_skill(Skill::Unbreakable).unwrap();
        c.gifts.push(2); // Chitinous Plates
        assert_eq!(c.defense_rating(), 6);
    }

    #[test]
    fn initiative_modifiers() {
        let mut c = character();
        assert_eq!(c.initiative(), 1);
        c.learn_skill(Skill::QuickDraw).unwrap();
        assert_eq!(c.initiative(), 3);
        c.drawbacks.push(Drawback::Slow);
        equip(&mut c, "Maul");
        assert_eq!(c.initiative(), -1);
        c.trauma.gain(9); // Broken
        assert_eq!(c.initiative(), -3);
    }

    #[test]
    fn vigor_never_goes_negative() {
        let mut c = character();
        c.attributes.set(Attribute::Con, 4);
        c.attributes.set(Attribute::Wil, 5);
        assert_eq!(c.max_vigor(), 0);
        c.attributes.set(Attribute::Con, 16);
        c.attributes.set(Attribute::Wil, 13);
        assert_eq!(c.max_vigor(), 3);
        c.learn_skill(Skill::Vigorous).unwrap();
        c.vigor_spent = 1;
        assert_eq!((c.max_vigor(), c.vigor()), (4, 3));
    }

    #[test]
    fn carrying_capacity_and_slots() {
        let mut c = character(); // STR 12
        assert_eq!((c.carrying_capacity(), c.item_slots()), (12, 12));
        c.learn_skill(Skill::HeavyLifting).unwrap();
        equip(&mut c, "Backpack");
        c.gifts.push(13); // Corpulent Storage
        assert_eq!(c.item_slots(), 17 + 10 + 5);
        c.trauma.gain(5); // Worn halves the capacity, not the backpack
        assert_eq!(c.item_slots(), 8 + 10 + 5);
    }

    #[test]
    fn encumbrance_kicks_in_past_the_slots() {
        let mut c = character(); // 12 slots
        c.inventory.rations = 120;
        assert!(!c.is_encumbered());
        c.inventory.rations = 121;
        assert!(c.is_encumbered());
    }

    #[test]
    fn move_anguish_and_injury_track() {
        let mut c = character(); // has Calm
        c.feats.push(Feat::Skirmisher);
        assert_eq!(c.move_speed(), 3);
        assert_eq!(c.anguish_check_modifier(), 2);
        c.drawbacks.push(Drawback::Cowardly);
        assert_eq!(c.extra_anguish(), 1);
        c.trauma.gain(11); // Shattered: also Shaken and Broken
        assert_eq!(c.anguish_check_modifier(), 1);
        assert_eq!(c.extra_anguish(), 2);
        assert_eq!(c.max_major_injuries(), 3);
    }

    #[test]
    fn five_blessings_is_too_far_gone() {
        let mut c = character();
        c.gifts = vec![1, 2, 3];
        c.stigmata = vec![1];
        assert!(!c.too_far_gone());
        c.stigmata.push(2);
        assert!(c.too_far_gone());
    }
}
