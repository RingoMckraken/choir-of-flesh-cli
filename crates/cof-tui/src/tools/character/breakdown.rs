//! Resolving a Breakdown (p.49). Most are pure roleplay; three change the sheet.

use cof_core::rules::Breakdown;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::anguish_text;
use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, TextInput, detail_box, page, render_menu};

const CLARITY: [&str; 2] = [
    "Defy it: lose all Anguish, but the next gain is doubled",
    "Embrace it: gain a Blessing of the Flesh, or a Prayer is resolved",
];

pub struct BreakdownScreen {
    breakdown: Breakdown,
    phobia: TextInput,
    choice: Menu,
}

impl BreakdownScreen {
    pub fn new(breakdown: Breakdown) -> Self {
        Self {
            breakdown,
            phobia: TextInput::new(60),
            choice: Menu::default(),
        }
    }

    fn resolve(&mut self, ctx: &mut AppContext) -> Transition {
        let name = self.breakdown.name();
        match self.breakdown {
            Breakdown::NewPhobia => {
                let fear = self.phobia.value().trim().to_owned();
                if fear.is_empty() {
                    ctx.error("Name the phobia first.");
                    return Transition::Stay;
                }
                if let Some(c) = ctx.character_mut() {
                    c.phobias.push(fear.clone());
                }
                ctx.record(
                    "Breakdown",
                    format!("{name}: a new, permanent phobia of {fear}."),
                );
            }
            Breakdown::MomentOfClarity => {
                if self.choice.selected(2) == Some(0) {
                    if let Some(c) = ctx.character_mut() {
                        c.anguish.defy_clarity();
                    }
                    ctx.record(
                        "Breakdown",
                        format!(
                            "{name}: Defied. All Anguish is gone, but the next gain is doubled."
                        ),
                    );
                } else {
                    ctx.record(
                        "Breakdown",
                        format!(
                            "{name}: Embraced. Gain a Blessing of the Flesh (record the Gift and \
                             Stigma under Details), or a Prayer is resolved."
                        ),
                    );
                }
            }
            Breakdown::TheVoiceWithin => {
                let Some(gain) = ctx.character_mut().map(|c| c.gain_permanent_anguish()) else {
                    return Transition::Pop;
                };
                ctx.record(
                    "Breakdown",
                    format!(
                        "{name}: a terrible truth, and a permanent point of Anguish. {}",
                        anguish_text::describe(&gain)
                    ),
                );
                if let Some(next) = gain.check.and_then(|c| c.breakdown) {
                    return Transition::Replace(Box::new(Self::new(next)));
                }
            }
            _ => ctx.record(
                "Breakdown",
                format!("{name}: {}", self.breakdown.description()),
            ),
        }
        ctx.save_campaign();
        Transition::Pop
    }
}

impl Screen for BreakdownScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &AppContext) {
        let help: &[(&str, &str)] = match self.breakdown {
            Breakdown::NewPhobia => &[("type", "the phobia"), ("enter", "done")],
            Breakdown::MomentOfClarity => &[("↑↓", "choose"), ("enter", "done")],
            _ => &[("enter", "done")],
        };
        let body = page(frame, area, "Breakdown", help);
        let [text, extra] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(body);
        detail_box(
            frame,
            text,
            self.breakdown.name(),
            vec![Line::styled(self.breakdown.description(), theme::text())],
        );
        match self.breakdown {
            Breakdown::NewPhobia => {
                let mut spans = vec![Span::styled("Afraid of: ", theme::key())];
                spans.extend(self.phobia.line().spans);
                detail_box(frame, extra, "The Phobia", vec![Line::from(spans)]);
            }
            Breakdown::MomentOfClarity => {
                let items = CLARITY
                    .iter()
                    .map(|c| Line::styled(*c, theme::text()))
                    .collect();
                render_menu(frame, extra, self.choice, items);
            }
            Breakdown::TheVoiceWithin => detail_box(
                frame,
                extra,
                "",
                vec![Line::styled(
                    "Press enter to take the permanent point of Anguish. It makes its own \
                     Anguish Check. Use the Oracle's Action & Theme for the truth you heard.",
                    theme::dim(),
                )],
            ),
            _ => detail_box(
                frame,
                extra,
                "",
                vec![Line::styled(
                    "Play it out, then press enter. It's written to the journal.",
                    theme::dim(),
                )],
            ),
        }
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        match self.breakdown {
            Breakdown::NewPhobia if key.code != KeyCode::Enter => {
                self.phobia.handle_key(key);
                Transition::Stay
            }
            Breakdown::MomentOfClarity if self.choice.handle_key(key, 2) => Transition::Stay,
            _ if key.code == KeyCode::Enter => self.resolve(ctx),
            _ => Transition::Stay,
        }
    }
}
