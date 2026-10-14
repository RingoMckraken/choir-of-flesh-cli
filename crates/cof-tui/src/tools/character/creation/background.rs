use cof_core::character::{Lore, occupation, occupations};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::theme;
use crate::widgets::{detail_box, render_menu};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Row {
    Lore(Lore),
    Occupation,
    OccupationSkill,
}

const ROWS: [Row; 5] = [
    Row::Lore(Lore::Sin),
    Row::Lore(Lore::Shard),
    Row::Lore(Lore::Doom),
    Row::Occupation,
    Row::OccupationSkill,
];

fn label(row: Row) -> &'static str {
    match row {
        Row::Lore(lore) => lore.name(),
        Row::Occupation => "Former Occupation",
        Row::OccupationSkill => "Occupation Skill",
    }
}

fn value(screen: &CreationScreen, row: Row) -> Option<u8> {
    let d = &screen.draft;
    match row {
        Row::Lore(Lore::Sin) => d.sin,
        Row::Lore(Lore::Shard) => d.shard,
        Row::Lore(Lore::Doom) => d.doom,
        Row::Occupation => d.occupation,
        _ => None,
    }
}

fn set(screen: &mut CreationScreen, row: Row, roll: u8) {
    let d = &mut screen.draft;
    match row {
        Row::Lore(Lore::Sin) => d.sin = Some(roll),
        Row::Lore(Lore::Shard) => d.shard = Some(roll),
        Row::Lore(Lore::Doom) => d.doom = Some(roll),
        Row::Occupation => d.set_occupation(roll),
        _ => {}
    }
}

fn roll(screen: &mut CreationScreen, row: Row) -> Option<String> {
    let roll = match row {
        Row::Lore(lore) => lore.roll(),
        Row::Occupation => occupations().roll().roll,
        Row::OccupationSkill => return None,
    };
    set(screen, row, roll);
    Some(format!("D10 = {roll}"))
}

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    if screen.menu.handle_key(key, ROWS.len()) {
        return;
    }
    let row = ROWS[screen.selected(ROWS.len())];
    match key.code {
        KeyCode::Left | KeyCode::Right => {
            let step: i16 = if key.code == KeyCode::Left { -1 } else { 1 };
            if row == Row::OccupationSkill {
                let options = screen.draft.occupation_skill_options();
                if options.is_empty() {
                    return;
                }
                let current = screen.draft.occupation_skill;
                let index = current
                    .and_then(|s| options.iter().position(|&o| o == s))
                    .map_or(if step > 0 { 0 } else { -1 }, |i| i as i16 + step);
                let pick = options[index.rem_euclid(options.len() as i16) as usize];
                screen.note = screen
                    .draft
                    .set_occupation_skill(pick)
                    .err()
                    .map(|e| format!("Can't: {e}."));
            } else {
                let current = value(screen, row).map_or(if step > 0 { 0 } else { 1 }, i16::from);
                let next = (current - 1 + step).rem_euclid(10) + 1;
                set(screen, row, next as u8);
            }
        }
        KeyCode::Char('r') => screen.note = roll(screen, row),
        KeyCode::Char('R') => {
            let mut rolled = Vec::new();
            for row in ROWS {
                if row != Row::OccupationSkill && value(screen, row).is_none() {
                    roll(screen, row);
                    rolled.push(label(row));
                }
            }
            screen.note = Some(if rolled.is_empty() {
                "Everything is already set.".into()
            } else {
                format!("Rolled {}.", rolled.join(", "))
            });
        }
        _ => {}
    }
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    let [list, hint] = Layout::vertical([Constraint::Length(6), Constraint::Min(0)]).areas(left);
    let items = ROWS
        .iter()
        .map(|&row| {
            let shown = match row {
                Row::Lore(lore) => value(screen, row).map(|r| lore.entry(r).title.clone()),
                Row::Occupation => value(screen, row).map(|r| occupation(r).title.clone()),
                Row::OccupationSkill => screen.draft.occupation_skill.map(|s| s.name().to_owned()),
            };
            Line::from(vec![
                Span::styled(format!("{:<24}", label(row)), theme::text()),
                Span::styled(
                    format!("◀ {} ▶", shown.as_deref().unwrap_or("—")),
                    theme::key(),
                ),
            ])
        })
        .collect();
    render_menu(frame, list, screen.menu, items);
    detail_box(
        frame,
        hint,
        "",
        vec![Line::styled(
            "Who you were before the end (p.18–25). Roll with r, or R for everything still \
             empty, or choose with ←→. Your Former Occupation gives one Skill from its list.",
            theme::dim(),
        )],
    );

    let row = ROWS[screen.selected(ROWS.len())];
    let (title, lines) = match (row, screen.draft.occupation) {
        (Row::Lore(lore), _) => match value(screen, row) {
            Some(r) => {
                let e = lore.entry(r);
                (
                    e.title.clone(),
                    vec![Line::styled(e.text.clone(), theme::text())],
                )
            }
            None => (
                String::new(),
                vec![Line::styled("Not yet chosen.", theme::dim())],
            ),
        },
        (Row::Occupation | Row::OccupationSkill, Some(r)) => {
            let o = occupation(r);
            let skills: Vec<_> = o.skills.iter().map(|s| s.name()).collect();
            (
                o.title.clone(),
                vec![
                    Line::styled(o.text.clone(), theme::text()),
                    Line::default(),
                    Line::styled(format!("Skills: {}", skills.join(", ")), theme::key()),
                    Line::default(),
                    Line::styled(format!("Trouble: {}", o.trouble), theme::dim()),
                ],
            )
        }
        (_, None) => (
            String::new(),
            vec![Line::styled("Not yet chosen.", theme::dim())],
        ),
    };
    detail_box(frame, right, &title, lines);
}
