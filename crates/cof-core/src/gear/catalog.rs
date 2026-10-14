use std::sync::LazyLock;

use super::types::{GearItem, GearKind};

/// The bundled gear catalog (p.209–216), parsed on first use.
static CATALOG: LazyLock<Vec<GearItem>> = LazyLock::new(|| {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/gear/catalog.ron"
    ));
    ron::from_str(source).unwrap_or_else(|e| panic!("bundled gear catalog is invalid: {e}"))
});

/// Every item that can be bought, in book order.
pub fn catalog() -> &'static [GearItem] {
    &CATALOG
}

/// Looks an item up by its exact name.
pub fn find(name: &str) -> Option<&'static GearItem> {
    CATALOG.iter().find(|item| item.name == name)
}

/// Weapons, each needing its own Proficiency (p.14).
pub fn weapons() -> impl Iterator<Item = &'static GearItem> {
    CATALOG
        .iter()
        .filter(|i| matches!(i.kind, GearKind::Weapon(_)))
}

/// Armor and shields that need a Proficiency (helmets don't, p.14).
pub fn proficiency_armor() -> impl Iterator<Item = &'static GearItem> {
    CATALOG
        .iter()
        .filter(|i| i.armor().is_some_and(|a| a.needs_proficiency()))
}

/// The starting kit's armor (p.12).
pub const GAMBESON: &str = "Gambeson";
/// Rations, the currency and food (p.208). Light, like the book says.
pub const RATIONS: &str = "Rations";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gear::{ArmorSlot, WeaponTrait, Weight};

    #[test]
    fn catalog_loads_with_every_section() {
        assert_eq!(weapons().count(), 14);
        let armor: Vec<_> = catalog().iter().filter_map(|i| i.armor()).collect();
        assert_eq!(armor.len(), 7);
        assert_eq!(
            armor.iter().filter(|a| a.slot == ArmorSlot::Helmet).count(),
            2
        );
        let names: Vec<_> = proficiency_armor().map(|i| i.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Gambeson",
                "Chainmail",
                "Scale Armor",
                "Light Shield",
                "Heavy Shield"
            ]
        );
        assert_eq!(catalog().len(), 14 + 7 + 22 + 3);
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<_> = catalog().iter().map(|i| &i.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), catalog().len());
    }

    #[test]
    fn spot_checks_against_the_rulebook() {
        let crossbow = find("Crossbow").unwrap();
        assert_eq!(crossbow.cost, 25);
        assert!(
            crossbow
                .weapon()
                .unwrap()
                .traits
                .contains(&WeaponTrait::Ranged(4))
        );
        let scale = find("Scale Armor").unwrap().armor().unwrap();
        assert_eq!((scale.dr, scale.min_str), (3, Some(14)));
        assert_eq!(
            find("Heavy Shield")
                .unwrap()
                .armor()
                .unwrap()
                .defense_mastery,
            2
        );
        assert_eq!(find("Tent").unwrap().weight, Weight::Heavy);
        assert_eq!(find(GAMBESON).unwrap().cost, 30);
        assert!(matches!(find("Backpack").unwrap().kind, GearKind::Backpack));
    }
}
