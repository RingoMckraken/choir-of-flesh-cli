use cof_core::gear::{self, GearItem};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::theme;
use crate::tools::character::gear_text;
use crate::widgets::{detail_box, render_menu};

const RULE: &str = "One weapon and one armor Proficiency (p.14). Each weapon and armor needs \
    its own. Without it you attack, or make defensive checks, with Disadvantage. Your free \
    Gambeson is armor too. Helmets need no Proficiency. More can be bought later with DP.";

fn options(row: usize) -> Vec<&'static GearItem> {
    if row == 0 {
        gear::weapons().collect()
    } else {
        gear::proficiency_armor().collect()
    }
}

fn current(screen: &CreationScreen, row: usize) -> Option<&String> {
    if row == 0 {
        screen.draft.weapon_proficiency.as_ref()
    } else {
        screen.draft.armor_proficiency.as_ref()
    }
}

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    if screen.menu.handle_key(key, 2) {
        return;
    }
    let step: i32 = match key.code {
        KeyCode::Left => -1,
        KeyCode::Right => 1,
        _ => return,
    };
    let row = screen.selected(2);
    let options = options(row);
    let index = current(screen, row)
        .and_then(|name| options.iter().position(|g| &g.name == name))
        .map_or(if step > 0 { 0 } else { -1 }, |i| i as i32 + step);
    let pick = options[index.rem_euclid(options.len() as i32) as usize]
        .name
        .clone();
    if row == 0 {
        screen.draft.weapon_proficiency = Some(pick);
    } else {
        screen.draft.armor_proficiency = Some(pick);
    }
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    let [list, rule] = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(left);
    let rows = ["Weapon", "Armor"]
        .iter()
        .enumerate()
        .map(|(row, label)| {
            let value = current(screen, row).map_or("choose", |s| s.as_str());
            Line::from(vec![
                Span::styled(format!("{label:<10}"), theme::text()),
                Span::styled(format!("◀ {value} ▶"), theme::key()),
            ])
        })
        .collect();
    render_menu(frame, list, screen.menu, rows);
    detail_box(frame, rule, "", vec![Line::styled(RULE, theme::dim())]);

    let row = screen.selected(2);
    let shown = current(screen, row).and_then(|name| gear::find(name));
    match shown {
        Some(item) => detail_box(frame, right, &item.name, gear_text::describe(item)),
        None => detail_box(
            frame,
            right,
            "",
            vec![Line::styled(
                "←→ to look through the options.",
                theme::dim(),
            )],
        ),
    }
}
