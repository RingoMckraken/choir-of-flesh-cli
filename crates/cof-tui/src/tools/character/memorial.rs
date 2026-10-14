//! The memorial of the fallen, and the way to raise the next Unbroken when no one is active.

use cof_core::character::Fallen;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::creation::CreationScreen;
use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, detail_box, page, render_menu};

#[derive(Default)]
pub struct MemorialScreen {
    menu: Menu,
}

fn fallen(ctx: &AppContext) -> &[Fallen] {
    ctx.campaign
        .as_ref()
        .map(|c| c.roster.fallen.as_slice())
        .unwrap_or_default()
}

/// With no active Unbroken, the first row raises one.
fn can_raise(ctx: &AppContext) -> bool {
    ctx.campaign.is_some() && ctx.character().is_none()
}

fn row_count(ctx: &AppContext) -> usize {
    fallen(ctx).len() + usize::from(can_raise(ctx))
}

impl MemorialScreen {
    /// The fallen on the selected row (newest first), if it is one.
    fn selected_fallen<'a>(&self, ctx: &'a AppContext) -> Option<&'a Fallen> {
        let index = self.menu.selected(row_count(ctx))?;
        let index = index.checked_sub(usize::from(can_raise(ctx)))?;
        fallen(ctx).iter().rev().nth(index)
    }
}

impl Screen for MemorialScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let raising = can_raise(ctx);
        let help: &[(&str, &str)] = if raising {
            &[("↑↓", "select"), ("enter", "raise"), ("esc", "back")]
        } else {
            &[("↑↓", "select"), ("esc", "back")]
        };
        let body = page(frame, area, "The Line of the Unbroken", help);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(body);
        let last = fallen(ctx).last();
        let mut items = Vec::new();
        if raising {
            let label = match last {
                Some(f) => format!("Raise a successor to {}", f.character.name),
                None => "Raise an Unbroken".to_owned(),
            };
            items.push(Line::styled(label, theme::key()));
        }
        for f in fallen(ctx).iter().rev() {
            items.push(Line::from(vec![
                Span::styled(format!("{:<22}", f.character.name), theme::text()),
                Span::styled(f.fate.describe(), theme::dim()),
            ]));
        }
        render_menu(frame, left, self.menu, items);

        let lines = match self.selected_fallen(ctx) {
            Some(f) => memorial_lines(f),
            None if fallen(ctx).is_empty() => vec![Line::styled(
                "No one has fallen yet. Raise the first Unbroken to defend the settlement.",
                theme::dim(),
            )],
            None => vec![Line::styled(
                "Someone must take up the fight. A successor is raised from the settlement's \
                 people, and may carry one heirloom from the one who fell.",
                theme::dim(),
            )],
        };
        detail_box(frame, right, "Memorial", lines);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.menu.handle_key(key, row_count(ctx)) {
            return Transition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Transition::Pop,
            KeyCode::Enter if can_raise(ctx) && self.menu.selected(row_count(ctx)) == Some(0) => {
                let successor_of = fallen(ctx).last().map(|f| f.character.name.clone());
                Transition::Push(Box::new(CreationScreen::new(successor_of)))
            }
            _ => Transition::Stay,
        }
    }
}

fn memorial_lines(f: &Fallen) -> Vec<Line<'static>> {
    let c = &f.character;
    let when = f.at.with_timezone(&chrono::Local).format("%b %-d %Y");
    vec![
        Line::styled(f.to_string(), theme::key()),
        Line::default(),
        Line::styled(format!("Fell on {when}."), theme::dim()),
        Line::styled(
            format!(
                "Once a {}, sworn to {}.",
                c.background.occupation().title,
                c.background.doom().title
            ),
            theme::text(),
        ),
        Line::styled(
            format!(
                "Humanity {}, Anguish {}, Trauma {} at the end.",
                c.humanity.current(),
                c.anguish.current(),
                c.trauma.current()
            ),
            theme::dim(),
        ),
    ]
}
