use cof_core::character::{Lore, Skill};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::{Confirm, CreationScreen};
use crate::theme;
use crate::widgets::{detail_box, render_menu};

const ROWS: usize = Skill::ALL.len();

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    if screen.menu.handle_key(key, ROWS) {
        return;
    }
    let skill = Skill::ALL[screen.selected(ROWS)];
    let result = match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => screen.draft.toggle_skill(skill),
        KeyCode::Char('b') => screen.draft.set_bonus_skill(skill),
        KeyCode::Char('g') if screen.draft.gift.is_none() => {
            screen.confirm = Some(Confirm::Gift);
            return;
        }
        _ => return,
    };
    screen.note = result.err().map(|e| format!("Can't: {e}."));
}

/// Rolls the Gift after the player confirmed.
pub(super) fn trade_for_gift(screen: &mut CreationScreen) {
    screen.note = Some(match screen.draft.trade_skill_for_gift() {
        Ok(gift) => format!(
            "Gift of the Unmade (D20 = {}): {}. Costs D4 = {} Humanity and 1 Anguish when \
             they're raised.",
            gift.gift,
            Lore::Gift.entry(gift.gift).title,
            gift.humanity_loss
        ),
        Err(e) => format!("Can't: {e}."),
    });
}

fn mark(screen: &CreationScreen, skill: Skill) -> (String, ratatui::style::Style) {
    let draft = &screen.draft;
    if draft.skills.contains(&skill) {
        ("✓".into(), theme::info())
    } else if draft.bonus_skills.contains(&skill) {
        (format!("bonus {}", skill.attribute().abbr()), theme::info())
    } else if draft.occupation_skill == Some(skill) {
        ("occupation".into(), theme::info())
    } else if let Err(e) = draft.skill_available(skill) {
        (format!("✗ {e}"), theme::dim())
    } else {
        (String::new(), theme::dim())
    }
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let items = Skill::ALL
        .iter()
        .map(|&s| {
            let (mark, style) = mark(screen, s);
            Line::from(vec![
                Span::styled(format!("{:<4}", s.attribute().abbr()), theme::dim()),
                Span::styled(format!("{:<26}", s.name()), theme::text()),
                Span::styled(mark, style),
            ])
        })
        .collect();
    render_menu(frame, left, screen.menu, items);

    let [description, summary] =
        Layout::vertical([Constraint::Percentage(40), Constraint::Min(0)]).areas(right);
    let skill = Skill::ALL[screen.selected(ROWS)];
    detail_box(
        frame,
        description,
        skill.name(),
        vec![Line::styled(skill.description(), theme::text())],
    );
    detail_box(frame, summary, "So Far", summary_lines(screen));
}

fn summary_lines(screen: &CreationScreen) -> Vec<Line<'static>> {
    let draft = &screen.draft;
    let mut lines = vec![Line::styled(
        format!(
            "Skills: {} of {}",
            draft.skills.len(),
            draft.general_skill_slots()
        ),
        theme::key(),
    )];
    for attribute in draft.bonus_skill_attributes() {
        let picked = draft
            .bonus_skill(attribute)
            .map_or("press b on one of its Skills", |s| s.name());
        lines.push(Line::styled(
            format!("Bonus {} Skill (16+): {picked}", attribute.abbr()),
            theme::key(),
        ));
    }
    match draft.gift {
        Some(gift) => lines.push(Line::styled(
            format!(
                "Gift: {} (costs {} Humanity, 1 Anguish)",
                Lore::Gift.entry(gift.gift).title,
                gift.humanity_loss
            ),
            theme::key(),
        )),
        None => lines.push(Line::styled(
            "g trades one pick for a rolled Gift of the Unmade (no Stigma, p.12).",
            theme::dim(),
        )),
    }
    lines.push(Line::default());
    lines.push(Line::styled(
        "Your Former Occupation adds one more Skill in the Background step.",
        theme::dim(),
    ));
    lines
}
