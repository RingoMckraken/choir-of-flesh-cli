//! Ending an Unbroken's story: death, being lost to the Choir or the Flesh, or retirement.

use cof_core::character::Fate;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, TextInput, detail_box, page, render_menu};

/// Why the screen opened: picked from the sheet, or forced by the rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    None,
    /// Trauma reached 13 (p.61).
    Trauma,
    /// Humanity reached 0 (p.47).
    Lost,
}

const DIED: usize = 0;
const CHOIR: usize = 1;
const FLESH: usize = 2;
const RETIRED: usize = 3;
const ROWS: usize = 4;

pub struct FallScreen {
    menu: Menu,
    cause: TextInput,
    preset: Preset,
    confirming: bool,
}

impl FallScreen {
    pub fn new(preset: Preset) -> Self {
        let mut cause = TextInput::new(60);
        let mut menu = Menu::default();
        match preset {
            Preset::Trauma => cause.set("Trauma"),
            Preset::Lost => {
                // Lands on "Lost to the Choir"; the player picks which.
                menu.handle_key(KeyEvent::from(KeyCode::Down), ROWS);
            }
            Preset::None => {}
        }
        Self {
            menu,
            cause,
            preset,
            confirming: false,
        }
    }

    fn fate(&self) -> Fate {
        match self.menu.selected(ROWS) {
            Some(CHOIR) => Fate::LostToTheChoir,
            Some(FLESH) => Fate::LostToTheFlesh,
            Some(RETIRED) => Fate::Retired,
            _ => Fate::Died {
                cause: self.cause.value().trim().to_owned(),
            },
        }
    }

    fn fall(&self, ctx: &mut AppContext) -> Transition {
        let fate = self.fate();
        let retired = fate == Fate::Retired;
        let Some(campaign) = ctx.campaign.as_mut() else {
            return Transition::Pop;
        };
        let Ok(memorial) = campaign.roster.fall(fate) else {
            return Transition::Pop;
        };
        let mut text = format!("Their story ends. {memorial}");
        if retired {
            text.push_str(
                " They rejoin the settlement: +1 Population (the Settlement Manager will \
                 track this).",
            );
        }
        let name = memorial.character.name.clone();
        ctx.record("Character", text);
        ctx.info(format!(
            "{name} is remembered. Someone must take up the fight."
        ));
        Transition::Pop
    }
}

impl Screen for FallScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let help: &[(&str, &str)] = if self.confirming {
            &[("y", "it's over"), ("any other key", "not yet")]
        } else {
            &[
                ("↑↓", "select"),
                ("type", "cause"),
                ("enter", "confirm"),
                ("esc", "back"),
            ]
        };
        let name = ctx.character().map_or("", |c| c.name.as_str());
        let body = page(frame, area, &format!("The End of {name}'s Story"), help);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(body);
        let mut died = vec![Span::styled("Died: ", theme::text())];
        died.extend(self.cause.line().spans);
        let items = vec![
            Line::from(died),
            Line::styled("Lost to the Choir (a Penitent)", theme::text()),
            Line::styled("Lost to the Flesh (one of the Unmade)", theme::text()),
            Line::styled("Retired (rejoins the settlement)", theme::text()),
        ];
        render_menu(frame, left, self.menu, items);

        let mut lines = Vec::new();
        match self.preset {
            Preset::Trauma => lines.push(Line::styled(
                "Trauma has reached 13: the body gives out (p.61).",
                theme::error(),
            )),
            Preset::Lost => lines.push(Line::styled(
                "Humanity has reached 0: they become an NPC, a Penitent or one of the \
                 Unmade (p.47). They may yet return as an enemy.",
                theme::error(),
            )),
            Preset::None => {}
        }
        lines.push(Line::styled(
            "They join the memorial. A successor can be raised from the settlement, and \
             may take one item they carried as an heirloom.",
            theme::dim(),
        ));
        if self.confirming {
            lines.push(Line::default());
            lines.push(Line::styled(
                format!("{}. This can't be undone. Press y.", self.fate().describe()),
                theme::error(),
            ));
        }
        detail_box(frame, right, "", lines);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.confirming {
            self.confirming = false;
            return if key.code == KeyCode::Char('y') {
                self.fall(ctx)
            } else {
                Transition::Stay
            };
        }
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                self.menu.handle_key(key, ROWS);
            }
            KeyCode::Esc => return Transition::Pop,
            KeyCode::Enter => self.confirming = true,
            _ if self.menu.selected(ROWS) == Some(DIED) => {
                self.cause.handle_key(key);
            }
            _ => {}
        }
        Transition::Stay
    }
}
