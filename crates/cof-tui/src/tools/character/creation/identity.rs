use cof_core::character::{Lore, Relationship};
use cof_core::oracle::name_table;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::theme;
use crate::widgets::{detail_box, render_menu};

const NAME: usize = 0;
const RANDOM_NAME: usize = 1;
const BOND: usize = 2;
const BOND_WITH: usize = 3;
const ROWS: usize = 4;

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    let row = screen.selected(ROWS);
    // Letters are for typing here, so only the arrows move the selection.
    if matches!(key.code, KeyCode::Up | KeyCode::Down) {
        screen.menu.handle_key(key, ROWS);
        return;
    }
    match row {
        NAME => {
            if screen.name.handle_key(key) {
                screen.draft.name = screen.name.value().to_owned();
            }
        }
        RANDOM_NAME => match key.code {
            KeyCode::Left | KeyCode::Right => screen.female_names = !screen.female_names,
            KeyCode::Enter | KeyCode::Char(' ') => {
                let roll = name_table().roll();
                let name = if screen.female_names {
                    &roll.value.female
                } else {
                    &roll.value.male
                };
                screen.name.set(name.clone());
                screen.draft.name = name.clone();
                screen.note = Some(format!("D100 = {}: {name}", roll.roll));
            }
            _ => {}
        },
        BOND => {
            let current = screen.draft.relationship.as_ref().map(|r| r.roll);
            let next = match key.code {
                // Cycles none → 1 … 10 → none.
                KeyCode::Right => match current {
                    None => Some(1),
                    Some(10) => None,
                    Some(r) => Some(r + 1),
                },
                KeyCode::Left => match current {
                    None => Some(10),
                    Some(1) => None,
                    Some(r) => Some(r - 1),
                },
                KeyCode::Char('r') | KeyCode::Enter => {
                    let roll = Lore::Relationship.roll();
                    screen.note = Some(format!("D10 = {roll}"));
                    Some(roll)
                }
                _ => return,
            };
            screen.draft.relationship = next.map(|roll| Relationship {
                roll,
                with: screen.bond_with.value().to_owned(),
            });
        }
        BOND_WITH => {
            if screen.bond_with.handle_key(key)
                && let Some(r) = screen.draft.relationship.as_mut()
            {
                r.with = screen.bond_with.value().to_owned();
            }
        }
        _ => {}
    }
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [list, hint] = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).areas(left);
    let label = |text: &str| Span::styled(format!("{text:<16}"), theme::text());
    let column = if screen.female_names {
        "Female"
    } else {
        "Male"
    };
    let bond = screen
        .draft
        .relationship
        .as_ref()
        .map_or("None".to_owned(), |r| {
            Lore::Relationship.entry(r.roll).title.clone()
        });
    let mut name_line = vec![label("Name")];
    name_line.extend(screen.name.line().spans);
    let mut with_line = vec![label("Bond with")];
    with_line.extend(screen.bond_with.line().spans);
    let items = vec![
        Line::from(name_line),
        Line::from(vec![
            label("Random name"),
            Span::styled(format!("◀ {column} ▶  enter rolls"), theme::key()),
        ]),
        Line::from(vec![
            label("Relationship"),
            Span::styled(format!("◀ {bond} ▶"), theme::key()),
        ]),
        Line::from(with_line),
    ];
    render_menu(frame, list, screen.menu, items);
    let successor = match &screen.successor_of {
        Some(name) => format!(" A successor's bond is often with the one who came before: {name}."),
        None => String::new(),
    };
    detail_box(
        frame,
        hint,
        "",
        vec![Line::styled(
            format!(
                "Name them, or roll on the names table (p.26). A Relationship (p.29) is \
                 optional: a bond with another Unbroken or someone in the settlement.{successor}"
            ),
            theme::dim(),
        )],
    );

    let lines = match &screen.draft.relationship {
        Some(r) => vec![Line::styled(
            Lore::Relationship.entry(r.roll).text.clone(),
            theme::text(),
        )],
        None => vec![Line::styled("No bond chosen.", theme::dim())],
    };
    detail_box(frame, right, &bond, lines);
}
