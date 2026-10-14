//! The creation wizard (p.12, p.224): one step per screen, roll or choose everywhere.
//! The core [`Draft`] holds every choice and says what's missing; this is just its face.

mod attributes;
mod background;
mod feats;
mod gear;
mod identity;
mod proficiencies;
mod review;
mod skills;
mod view;

use cof_core::character::Attribute;
use cof_core::character::creation::{Draft, Step};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::widgets::{Menu, TextInput};

/// A yes/no question waiting on the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirm {
    /// Throw the draft away.
    Cancel,
    /// Trade a Skill for a rolled Gift of the Unmade (can't be undone).
    Gift,
}

pub struct CreationScreen {
    draft: Draft,
    step: Step,
    /// The selected row in the current step.
    menu: Menu,
    /// Set when raising a successor: the fallen Unbroken's name.
    successor_of: Option<String>,
    /// The Heirloom: an index into the newest fallen's items, if one was picked.
    heirloom: Option<usize>,
    name: TextInput,
    bond_with: TextInput,
    /// Which column of the names table to roll (p.26): female if true.
    female_names: bool,
    confirm: Option<Confirm>,
    /// The last roll or refusal, shown in the step.
    note: Option<String>,
}

impl CreationScreen {
    /// A new draft. `successor_of` names the fallen Unbroken this one follows, if any.
    pub fn new(successor_of: Option<String>) -> Self {
        let mut draft = Draft::new();
        // Start with the rolls in the order they came, so the step is already valid and
        // the player only swaps what they want to.
        for (i, a) in Attribute::ALL.into_iter().enumerate() {
            draft.assign(a, i);
        }
        let mut bond_with = TextInput::new(40);
        if let Some(name) = &successor_of {
            bond_with.set(name.clone());
        }
        Self {
            draft,
            step: Step::Attributes,
            menu: Menu::default(),
            successor_of,
            heirloom: None,
            name: TextInput::new(40),
            bond_with,
            female_names: false,
            confirm: None,
            note: None,
        }
    }

    fn step_index(&self) -> usize {
        Step::ALL.iter().position(|&s| s == self.step).unwrap_or(0)
    }

    fn go_to(&mut self, index: usize) {
        self.step = Step::ALL[index.min(Step::ALL.len() - 1)];
        self.menu = Menu::default();
        self.note = None;
    }

    fn next_step(&mut self) {
        self.go_to(self.step_index() + 1);
    }

    fn previous_step(&mut self) {
        self.go_to(self.step_index().saturating_sub(1));
    }

    fn selected(&self, len: usize) -> usize {
        self.menu.selected(len).unwrap_or(0)
    }

    fn confirmed(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        let question = self.confirm.take();
        if key.code != KeyCode::Char('y') {
            return Transition::Stay;
        }
        match question {
            Some(Confirm::Cancel) => {
                ctx.info("The draft is torn up.");
                Transition::Pop
            }
            Some(Confirm::Gift) => {
                skills::trade_for_gift(self);
                Transition::Stay
            }
            None => Transition::Stay,
        }
    }
}

impl Screen for CreationScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        view::draw(self, frame, area, ctx);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.confirm.is_some() {
            return self.confirmed(key, ctx);
        }
        match key.code {
            KeyCode::Tab => {
                self.next_step();
                return Transition::Stay;
            }
            KeyCode::BackTab => {
                self.previous_step();
                return Transition::Stay;
            }
            KeyCode::Esc => {
                self.confirm = Some(Confirm::Cancel);
                return Transition::Stay;
            }
            _ => {}
        }
        match self.step {
            Step::Attributes => attributes::key(self, key),
            Step::Proficiencies => proficiencies::key(self, key),
            Step::Feats => feats::key(self, key),
            Step::Skills => skills::key(self, key),
            Step::Background => background::key(self, key),
            Step::Identity => identity::key(self, key),
            Step::Gear => gear::key(self, key),
            Step::Review => return review::key(self, key, ctx),
        }
        Transition::Stay
    }
}
