use cof_core::character::{Attribute, modifier};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::theme;
use crate::widgets::{detail_box, render_menu};

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    if screen.menu.handle_key(key, Attribute::ALL.len()) {
        return;
    }
    let step: i32 = match key.code {
        KeyCode::Left => -1,
        KeyCode::Right => 1,
        _ => return,
    };
    let attribute = Attribute::ALL[screen.selected(Attribute::ALL.len())];
    let current = screen.draft.assigned[attribute as usize].unwrap_or(0) as i32;
    let next = (current + step).rem_euclid(6) as usize;
    screen.draft.assign(attribute, next);
    screen.draft.prune_bonus_skills();
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    let [list, intro] = Layout::vertical([Constraint::Length(7), Constraint::Min(0)]).areas(left);

    let draft = &screen.draft;
    let rows = Attribute::ALL
        .iter()
        .map(|&a| {
            let assigned = draft.assigned[a as usize];
            let score = assigned.map(|i| draft.rolls[i].total());
            let text = match (score, assigned) {
                (Some(s), Some(i)) => {
                    format!("{s:>2} ({:+})   ◀ roll {} ▶", modifier(s), i + 1)
                }
                _ => "◀ assign ▶".into(),
            };
            Line::from(vec![
                Span::styled(format!("{:<14}", a.name()), theme::text()),
                Span::styled(text, theme::key()),
            ])
        })
        .collect();
    render_menu(frame, list, screen.menu, rows);
    detail_box(
        frame,
        intro,
        "",
        vec![Line::styled(
            "Six rolls of 4D6, dropping the lowest die. Give each Attribute the roll you \
             want (p.13).",
            theme::dim(),
        )],
    );

    let mut lines = Vec::new();
    for (i, roll) in draft.rolls.iter().enumerate() {
        let mut spans = vec![Span::styled(format!("Roll {}: ", i + 1), theme::dim())];
        let mut dropped = false;
        for &die in &roll.dice {
            let style = if !dropped && die == roll.dropped() {
                dropped = true;
                theme::dim().add_modifier(Modifier::CROSSED_OUT)
            } else {
                theme::text()
            };
            spans.push(Span::styled(format!("{die} "), style));
        }
        spans.push(Span::styled(format!("= {:>2}", roll.total()), theme::key()));
        lines.push(Line::from(spans));
    }
    if let Some(attrs) = draft.attributes() {
        let m = |a| attrs.modifier(a);
        lines.push(Line::default());
        lines.push(Line::styled("With these scores", theme::title()));
        let derived = [
            ("Carrying Capacity", attrs.score(Attribute::Str).to_string()),
            ("Defense Rating", format!("{:+} + armor", m(Attribute::Dex))),
            ("Initiative", format!("{:+}", m(Attribute::Dex))),
            (
                "Vigor",
                (m(Attribute::Con) + m(Attribute::Wil)).max(0).to_string(),
            ),
        ];
        for (label, value) in derived {
            lines.push(Line::from(vec![
                Span::styled(format!("{label:<18}"), theme::dim()),
                Span::styled(value, theme::text()),
            ]));
        }
        let bonus: Vec<_> = attrs
            .bonus_skill_attributes()
            .iter()
            .map(|a| a.abbr())
            .collect();
        if !bonus.is_empty() {
            lines.push(Line::styled(
                format!("16+ in {}: a bonus Skill from each list.", bonus.join(", ")),
                theme::info(),
            ));
        }
    }
    detail_box(frame, right, "The Dice", lines);
}
