use cof_core::character::creation::STARTING_RATIONS;
use cof_core::gear::{self, GearItem};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::theme;
use crate::tools::character::gear_text;
use crate::widgets::{detail_box, render_menu};

fn rows() -> usize {
    1 + gear::catalog().len()
}

/// The shop item on a row (row 0 is the free weapon).
fn shop_item(row: usize) -> Option<&'static GearItem> {
    row.checked_sub(1).and_then(|i| gear::catalog().get(i))
}

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    if screen.menu.handle_key(key, rows()) {
        return;
    }
    let row = screen.selected(rows());
    let result = match (row, key.code) {
        (0, KeyCode::Left | KeyCode::Right) => {
            let weapons: Vec<_> = gear::weapons().collect();
            let step: i32 = if key.code == KeyCode::Left { -1 } else { 1 };
            let index = screen
                .draft
                .starting_weapon
                .as_ref()
                .and_then(|w| weapons.iter().position(|g| &g.name == w))
                .map_or(if step > 0 { 0 } else { -1 }, |i| i as i32 + step);
            let pick = &weapons[index.rem_euclid(weapons.len() as i32) as usize].name;
            screen.draft.set_starting_weapon(pick)
        }
        (_, KeyCode::Char('+' | '=') | KeyCode::Enter) => match shop_item(row) {
            Some(item) => screen.draft.buy(&item.name),
            None => return,
        },
        (_, KeyCode::Char('-' | '_') | KeyCode::Backspace) => {
            if let Some(item) = shop_item(row) {
                screen.draft.unbuy(&item.name);
            }
            Ok(())
        }
        _ => return,
    };
    screen.note = result.err().map(|_| "Not enough Rations for that.".into());
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let draft = &screen.draft;
    let mut items = vec![Line::from(vec![
        Span::styled(format!("{:<28}", "Free weapon"), theme::text()),
        Span::styled(
            format!(
                "◀ {} ▶",
                draft.starting_weapon.as_deref().unwrap_or("choose")
            ),
            theme::key(),
        ),
    ])];
    items.extend(gear::catalog().iter().map(|item| {
        let in_cart = draft.in_cart(&item.name);
        let style = if item.cost > draft.rations_left() && in_cart == 0 {
            theme::dim()
        } else {
            theme::text()
        };
        Line::from(vec![
            Span::styled(format!("{:<28}", item.name), style),
            Span::styled(format!("{:>4} ☈", item.cost), theme::dim()),
            Span::styled(
                if in_cart > 0 {
                    format!("   × {in_cart}")
                } else {
                    String::new()
                },
                theme::info(),
            ),
        ])
    }));
    render_menu(frame, left, screen.menu, items);

    let [description, budget] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(right);
    let row = screen.selected(rows());
    let shown = shop_item(row).or_else(|| draft.starting_weapon.as_deref().and_then(gear::find));
    match shown {
        Some(item) => detail_box(frame, description, &item.name, gear_text::describe(item)),
        None => detail_box(frame, description, "", Vec::new()),
    }

    let mut lines = vec![
        Line::styled(
            format!(
                "Rations left: {} of {STARTING_RATIONS}",
                draft.rations_left()
            ),
            theme::key(),
        ),
        Line::styled(
            "Free: your weapon of choice and a Gambeson (p.12). Unspent Rations stay in \
             your pack as food (10 to an Item Slot).",
            theme::dim(),
        ),
    ];
    if !draft.cart.is_empty() {
        lines.push(Line::default());
        for (name, qty) in &draft.cart {
            lines.push(Line::styled(format!("{qty} × {name}"), theme::text()));
        }
    }
    detail_box(frame, budget, "Budget", lines);
}
