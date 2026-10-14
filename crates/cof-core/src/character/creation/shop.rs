use super::Draft;
use super::choices::ChoiceError;
use crate::gear::{self, GearKind};

/// Rations every new character starts with, to spend on gear or keep (p.12).
pub const STARTING_RATIONS: u16 = 30;

impl Draft {
    /// Picks the free starting weapon (any weapon, p.12). Proficiency isn't checked: a
    /// character can carry a weapon they fight badly with.
    pub fn set_starting_weapon(&mut self, name: &str) -> Result<(), ChoiceError> {
        if !gear::weapons().any(|w| w.name == name) {
            return Err(ChoiceError::NotAllowed);
        }
        self.starting_weapon = Some(name.to_owned());
        Ok(())
    }

    /// Rations spent in the shop so far.
    pub fn spent(&self) -> u16 {
        self.cart
            .iter()
            .filter_map(|(name, qty)| gear::find(name).map(|g| g.cost * qty))
            .sum()
    }

    /// Rations left to keep: the leftovers start in the character's pack.
    pub fn rations_left(&self) -> u16 {
        STARTING_RATIONS.saturating_sub(self.spent())
    }

    /// How many of an item are in the cart.
    pub fn in_cart(&self, name: &str) -> u16 {
        self.cart
            .iter()
            .find(|(n, _)| n == name)
            .map_or(0, |(_, q)| *q)
    }

    /// Buys one of a catalog item, if the Rations cover it.
    pub fn buy(&mut self, name: &str) -> Result<(), ChoiceError> {
        let item = gear::find(name).ok_or(ChoiceError::NotAllowed)?;
        if matches!(item.kind, GearKind::Backpack) && self.in_cart(name) >= 2 {
            return Err(ChoiceError::Full);
        }
        if item.cost > self.rations_left() {
            return Err(ChoiceError::Full);
        }
        match self.cart.iter_mut().find(|(n, _)| n == name) {
            Some((_, qty)) => *qty += 1,
            None => self.cart.push((name.to_owned(), 1)),
        }
        Ok(())
    }

    /// Puts one back.
    pub fn unbuy(&mut self, name: &str) {
        if let Some(i) = self.cart.iter().position(|(n, _)| n == name) {
            self.cart[i].1 -= 1;
            if self.cart[i].1 == 0 {
                self.cart.remove(i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_budget_is_thirty_rations() {
        let mut d = Draft::new();
        d.buy("Backpack").unwrap(); // 20
        d.buy("Torch").unwrap(); // 2
        d.buy("Torch").unwrap(); // 2
        assert_eq!(
            (d.spent(), d.rations_left(), d.in_cart("Torch")),
            (24, 6, 2)
        );
        d.buy("Medical Supplies").unwrap(); // 5
        assert_eq!(d.buy("Rope"), Err(ChoiceError::Full), "5 > 1 left");
        d.unbuy("Torch");
        d.unbuy("Torch");
        assert_eq!((d.in_cart("Torch"), d.rations_left()), (0, 5));
        assert_eq!(d.buy("Spangenhelm"), Err(ChoiceError::Full));
        assert_eq!(d.buy("Rubber Duck"), Err(ChoiceError::NotAllowed));
    }

    #[test]
    fn any_weapon_can_be_the_free_one() {
        let mut d = Draft::new();
        d.set_starting_weapon("Greatsword").unwrap();
        assert_eq!(
            d.set_starting_weapon("Gambeson"),
            Err(ChoiceError::NotAllowed)
        );
        assert_eq!(d.starting_weapon.as_deref(), Some("Greatsword"));
    }
}
