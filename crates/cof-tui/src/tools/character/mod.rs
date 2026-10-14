//! Character Creator & Sheet (p.12–39): the settlement's line of Unbroken.
//!
//! The hub opens [`CharacterTool`], which shows the active Unbroken's sheet, or the memorial
//! (and the way to raise someone new) when no one is active. Every sub-screen pops back to
//! it, so a fall lands on the memorial and a finished creation lands on the new sheet.

mod advance;
mod anguish_text;
mod breakdown;
mod creation;
mod details;
mod fall;
mod gear_text;
mod inventory;
mod memorial;
mod sheet;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use memorial::MemorialScreen;
use sheet::SheetScreen;

#[derive(Default)]
pub struct CharacterTool {
    sheet: SheetScreen,
    memorial: MemorialScreen,
}

impl Screen for CharacterTool {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        if ctx.character().is_some() {
            self.sheet.draw(frame, area, ctx);
        } else {
            self.memorial.draw(frame, area, ctx);
        }
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if ctx.character().is_some() {
            self.sheet.handle_key(key, ctx)
        } else {
            self.memorial.handle_key(key, ctx)
        }
    }
}
