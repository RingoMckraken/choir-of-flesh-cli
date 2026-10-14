use cof_core::character::{Drawback, Feat};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::theme;
use crate::widgets::{detail_box, render_menu};

const ROWS: usize = Feat::ALL.len() + Drawback::ALL.len();

/// The row's Feat, or the Drawback for rows after the Feats.
fn row(index: usize) -> Result<Feat, Drawback> {
    match Feat::ALL.get(index) {
        Some(&feat) => Ok(feat),
        None => Err(Drawback::ALL[index - Feat::ALL.len()]),
    }
}

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent) {
    if screen.menu.handle_key(key, ROWS) {
        return;
    }
    let draft = &mut screen.draft;
    let result = match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => match row(screen.menu.selected(ROWS).unwrap_or(0)) {
            Ok(feat) => draft.toggle_feat(feat).map(|_| None),
            Err(drawback) => match draft
                .drawbacks
                .iter()
                .position(|p| p.drawback == Some(drawback))
            {
                Some(i) => {
                    draft.remove_drawback(i);
                    Ok(None)
                }
                None => draft.choose_drawback(drawback).map(|_| None),
            },
        },
        KeyCode::Char('r') => draft.roll_drawback().map(|pick| {
            let roll = pick.roll.unwrap_or_default();
            Some(match pick.drawback {
                Some(d) => format!("D12 = {roll}: {}. Take another Feat for it.", d.name()),
                None => format!("D12 = {roll}: No Drawback. You got lucky."),
            })
        }),
        KeyCode::Char('x') if !draft.drawbacks.is_empty() => {
            draft.remove_drawback(draft.drawbacks.len() - 1);
            Ok(Some("Took back the last Drawback pick.".into()))
        }
        _ => return,
    };
    screen.note = match result {
        Ok(note) => note,
        Err(e) => Some(format!("Can't: {e}.")),
    };
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    let draft = &screen.draft;
    let items = (0..ROWS)
        .map(|i| {
            let (kind, name, mark) = match row(i) {
                Ok(feat) => (
                    "Feat",
                    feat.name(),
                    draft.feats.contains(&feat).then(|| "✓".to_owned()),
                ),
                Err(d) => {
                    let pick = draft.drawbacks.iter().find(|p| p.drawback == Some(d));
                    let mark = pick.map(|p| match p.roll {
                        Some(roll) => format!("rolled {roll}"),
                        None => "chosen".to_owned(),
                    });
                    ("Drawback", d.name(), mark)
                }
            };
            Line::from(vec![
                Span::styled(format!("{kind:<9}"), theme::dim()),
                Span::styled(format!("{name:<21}"), theme::text()),
                Span::styled(mark.unwrap_or_default(), theme::info()),
            ])
        })
        .collect();
    render_menu(frame, left, screen.menu, items);

    let [description, summary] =
        Layout::vertical([Constraint::Percentage(45), Constraint::Min(0)]).areas(right);
    let (title, text) = match row(screen.selected(ROWS)) {
        Ok(f) => (f.name(), f.description()),
        Err(d) => (d.name(), d.description()),
    };
    detail_box(
        frame,
        description,
        title,
        vec![Line::styled(text, theme::text())],
    );

    let picks: Vec<String> = draft
        .drawbacks
        .iter()
        .map(|p| match (p.drawback, p.roll) {
            (Some(d), _) => d.name().to_owned(),
            (None, Some(roll)) => format!("lucky {roll}"),
            (None, None) => "none".to_owned(),
        })
        .collect();
    let lines = vec![
        Line::styled(
            format!("Feats: {} of {}", draft.feats.len(), draft.feat_allowance()),
            theme::key(),
        ),
        Line::styled(
            format!(
                "Drawbacks: {} of 3 tries{}",
                draft.drawbacks.len(),
                if picks.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", picks.join(", "))
                }
            ),
            theme::key(),
        ),
        Line::default(),
        Line::styled(
            "One free Feat. Each Drawback, rolled or chosen, earns another, up to 3 Feats \
             (p.12). A rolled 11–12 is no drawback at all. x takes back the last pick.",
            theme::dim(),
        ),
    ];
    detail_box(frame, summary, "So Far", lines);
}
