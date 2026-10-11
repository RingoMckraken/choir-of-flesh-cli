use cof_core::campaign::UsageTracker;
use cof_core::dice::{Die, UsageDie};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, TextInput, page, render_menu};

/// The sizes a new Usage Die can start at.
const SIZES: [Die; 6] = [Die::D4, Die::D6, Die::D8, Die::D10, Die::D12, Die::D20];

enum Mode {
    Browse,
    Adding { name: TextInput, size: usize },
    ConfirmRemove,
}

/// The campaign's named Usage Dice (p.46, p.90): add, roll, restock, remove.
pub(super) struct UsageDiceScreen {
    menu: Menu,
    mode: Mode,
}

impl Default for UsageDiceScreen {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            mode: Mode::Browse,
        }
    }
}

fn trackers(ctx: &AppContext) -> &[UsageTracker] {
    ctx.campaign
        .as_ref()
        .map(|c| c.usage_dice.as_slice())
        .unwrap_or_default()
}

impl UsageDiceScreen {
    fn selected_index(&self, ctx: &AppContext) -> Option<usize> {
        self.menu.selected(trackers(ctx).len())
    }

    /// Applies `change` to the selected tracker, journals its message, and saves.
    fn update(&self, ctx: &mut AppContext, change: impl FnOnce(&mut UsageTracker) -> String) {
        let Some(index) = self.selected_index(ctx) else {
            return;
        };
        let Some(campaign) = ctx.campaign.as_mut() else {
            return;
        };
        let text = change(&mut campaign.usage_dice[index]);
        campaign.record("Usage Dice", text.clone());
        ctx.save_campaign();
        if ctx.status.is_none() {
            ctx.info(text);
        }
    }

    fn browse_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.menu.handle_key(key, trackers(ctx).len()) {
            return Transition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Transition::Pop,
            KeyCode::Char('a') => {
                self.mode = Mode::Adding {
                    name: TextInput::new(30),
                    size: 1,
                }
            }
            KeyCode::Enter | KeyCode::Char('r') => self.update(ctx, |t| {
                let roll = t.roll();
                t.describe(&roll)
            }),
            KeyCode::Char('+') => self.update(ctx, |t| {
                t.restock();
                format!("{} restocked to UD{}.", t.name, t.die.die().sides())
            }),
            KeyCode::Char('x') if self.selected_index(ctx).is_some() => {
                self.mode = Mode::ConfirmRemove;
            }
            _ => {}
        }
        Transition::Stay
    }
}

impl Screen for UsageDiceScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let help: &[(&str, &str)] = match self.mode {
            Mode::Browse => &[
                ("enter", "roll"),
                ("a", "add"),
                ("+", "restock"),
                ("x", "remove"),
                ("esc", "back"),
            ],
            Mode::Adding { .. } => &[
                ("type", "name"),
                ("←→", "die"),
                ("enter", "add"),
                ("esc", "cancel"),
            ],
            Mode::ConfirmRemove => &[("y", "remove"), ("any other key", "keep")],
        };
        let body = page(frame, area, "Usage Dice", help);
        let [list, footer] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(body);

        let items: Vec<Line> = trackers(ctx)
            .iter()
            .map(|t| {
                let die = format!("UD{}", t.die.die().sides());
                let state = if t.depleted {
                    Span::styled("DEPLETED", theme::error())
                } else {
                    Span::styled(die, theme::key())
                };
                Line::from(vec![
                    Span::styled(format!("{:<32}", t.name), theme::text()),
                    state,
                ])
            })
            .collect();
        if items.is_empty() {
            let hint = "Track torches, arrows, or a ticking doom. Press a to add one.";
            frame.render_widget(Paragraph::new(Line::styled(hint, theme::dim())), list);
        } else {
            render_menu(frame, list, self.menu, items);
        }

        let footer_line = match &self.mode {
            Mode::Browse => Line::styled(
                "Roll 1–2 to step down (D20 → D12 → … → D4). 1–2 on a D4: depleted (p.46).",
                theme::dim(),
            ),
            Mode::Adding { name, size } => Line::from(vec![
                Span::styled("New: ", theme::dim()),
                Span::styled(name.value().to_owned(), theme::text()),
                Span::styled("█  ", theme::dim()),
                Span::styled(format!("◀ UD{} ▶", SIZES[*size].sides()), theme::key()),
            ]),
            Mode::ConfirmRemove => Line::styled("Remove this tracker?", theme::error()),
        };
        frame.render_widget(Paragraph::new(footer_line), footer);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        match &mut self.mode {
            Mode::Browse => return self.browse_key(key, ctx),
            Mode::ConfirmRemove => {
                if key.code == KeyCode::Char('y')
                    && let Some(index) = self.selected_index(ctx)
                    && let Some(campaign) = ctx.campaign.as_mut()
                {
                    let removed = campaign.usage_dice.remove(index);
                    ctx.save_campaign();
                    ctx.info(format!("Removed {}.", removed.name));
                }
                self.mode = Mode::Browse;
            }
            Mode::Adding { name, size } => {
                if name.handle_key(key) {
                    return Transition::Stay;
                }
                match key.code {
                    KeyCode::Esc => self.mode = Mode::Browse,
                    KeyCode::Left => *size = size.saturating_sub(1),
                    KeyCode::Right => *size = (*size + 1).min(SIZES.len() - 1),
                    KeyCode::Enter if !name.value().trim().is_empty() => {
                        let die = UsageDie::new(SIZES[*size]).expect("SIZES excludes the D100");
                        let tracker = UsageTracker::new(name.value().trim(), die);
                        if let Some(campaign) = ctx.campaign.as_mut() {
                            campaign.usage_dice.push(tracker);
                            ctx.save_campaign();
                        }
                        self.mode = Mode::Browse;
                    }
                    _ => {}
                }
            }
        }
        Transition::Stay
    }
}
