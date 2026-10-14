//! The character sheet: trackers to adjust on the left, the whole Unbroken on the right.

mod input;
mod summary;
mod view;

use cof_core::character::{Character, Feat};
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::context::AppContext;
use crate::screen::Transition;
use crate::widgets::Menu;

/// One action on the sheet's menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Humanity,
    Anguish,
    Trauma,
    Vigor,
    Xp,
    Rumors,
    Rations,
    Rest,
    EndSession,
    Lucky,
    Defiance,
    Catharsis,
    Inventory,
    Advance,
    Details,
    Memorial,
    EndStory,
}

impl Row {
    fn label(self) -> &'static str {
        match self {
            Row::Humanity => "Humanity",
            Row::Anguish => "Anguish",
            Row::Trauma => "Trauma",
            Row::Vigor => "Vigor",
            Row::Xp => "Experience",
            Row::Rumors => "Rumors",
            Row::Rations => "Rations",
            Row::Rest => "Rest 6 hours",
            Row::EndSession => "End Session",
            Row::Lucky => "Lucky reroll",
            Row::Defiance => "Act of Defiance",
            Row::Catharsis => "Catharsis",
            Row::Inventory => "Inventory",
            Row::Advance => "Advance",
            Row::Details => "Details",
            Row::Memorial => "Memorial",
            Row::EndStory => "End their story",
        }
    }
}

/// The rows that apply to this character, in order.
fn rows(c: &Character) -> Vec<Row> {
    let mut rows = vec![
        Row::Humanity,
        Row::Anguish,
        Row::Trauma,
        Row::Vigor,
        Row::Xp,
        Row::Rumors,
        Row::Rations,
        Row::Rest,
        Row::EndSession,
    ];
    if c.has_feat(Feat::Lucky) {
        rows.push(Row::Lucky);
    }
    rows.push(Row::Defiance);
    if c.catharsis_available {
        rows.push(Row::Catharsis);
    }
    rows.extend([
        Row::Inventory,
        Row::Advance,
        Row::Details,
        Row::Memorial,
        Row::EndStory,
    ]);
    rows
}

/// The latest result, shown under the menu.
struct LastResult {
    title: &'static str,
    text: String,
}

#[derive(Default)]
pub(super) struct SheetScreen {
    menu: Menu,
    last: Option<LastResult>,
    /// Waiting for y to end the session.
    confirm_end_session: bool,
}

impl SheetScreen {
    fn selected_row(&self, ctx: &AppContext) -> Option<Row> {
        let rows = rows(ctx.character()?);
        Some(rows[self.menu.selected(rows.len())?])
    }

    /// Shows a result and writes it to the journal.
    fn report(&mut self, ctx: &mut AppContext, title: &'static str, text: String) {
        ctx.record("Character", text.clone());
        self.last = Some(LastResult { title, text });
    }
}

/// Drawn and driven by the character tool while there's an active Unbroken.
impl SheetScreen {
    pub fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        view::draw(self, frame, area, ctx);
    }

    pub fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        input::handle_key(self, key, ctx)
    }
}
