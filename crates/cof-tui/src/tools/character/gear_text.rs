use cof_core::gear::{ArmorSlot, GearItem, GearKind};
use ratatui::text::{Line, Span};

use crate::theme;

/// A catalog item's stats as lines: cost, weight, damage and traits, DR, notes.
pub fn describe(item: &GearItem) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![
        Span::styled(format!("{} ☈", item.cost), theme::key()),
        Span::styled(format!("   {}", item.weight.name()), theme::dim()),
    ])];
    match &item.kind {
        GearKind::Weapon(w) => {
            let damage: Vec<_> = w.damage.iter().map(|d| d.name()).collect();
            let traits: Vec<_> = w.traits.iter().map(|t| t.label()).collect();
            lines.push(field("Damage", damage.join(", ")));
            lines.push(field("Traits", traits.join(", ")));
        }
        GearKind::Armor(a) => {
            let slot = match a.slot {
                ArmorSlot::Padding => "Padding (worn under armor)",
                ArmorSlot::Body => "Body armor",
                ArmorSlot::Helmet => "Helmet (no Proficiency needed)",
                ArmorSlot::Shield => "Shield",
            };
            lines.push(field("Worn as", slot.to_owned()));
            if a.dr > 0 {
                lines.push(field("Defense", format!("+{} DR", a.dr)));
            }
            if a.defense_mastery > 0 {
                lines.push(field(
                    "Defense",
                    format!("+{} Mastery to DR checks", a.defense_mastery),
                ));
            }
            if let Some(str) = a.min_str {
                lines.push(field("Requires", format!("STR {str}+")));
            }
            if a.stealth_disadvantage {
                lines.push(field("Stealth", "Disadvantage".to_owned()));
            }
        }
        GearKind::Backpack | GearKind::Gear => {}
    }
    if !item.notes.is_empty() {
        lines.push(Line::styled(item.notes.clone(), theme::text()));
    }
    lines
}

fn field(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<9}"), theme::dim()),
        Span::styled(value, theme::text()),
    ])
}
