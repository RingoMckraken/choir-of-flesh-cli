use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::context::AppContext;

/// One screen of the app. Screens form a stack: the top one is drawn and gets the keys.
pub trait Screen {
    /// Draws the screen into `area`. Drawing only reads shared state.
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext);

    /// Handles a key press and says what should happen to the screen stack.
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition;
}

pub enum Transition {
    Stay,
    /// Open a screen on top of this one.
    Push(Box<dyn Screen>),
    /// Close this screen and open another in its place.
    Replace(Box<dyn Screen>),
    /// Close this screen, returning to the one below.
    Pop,
    Quit,
}
