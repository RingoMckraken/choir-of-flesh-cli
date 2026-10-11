use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{centered, page};

/// Placeholder for a tool that hasn't been built yet.
pub struct ComingSoon {
    name: &'static str,
    milestone: u8,
}

impl ComingSoon {
    pub fn new(name: &'static str, milestone: u8) -> Self {
        Self { name, milestone }
    }
}

impl Screen for ComingSoon {
    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &AppContext) {
        let body = page(
            frame,
            centered(area, 56, 8),
            self.name,
            &[("any key", "back")],
        );
        let text = vec![
            Line::styled("This tool hasn't been built yet.", theme::text()),
            Line::styled(
                format!("It arrives in Milestone {}.", self.milestone),
                theme::dim(),
            ),
        ];
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), body);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut AppContext) -> Transition {
        Transition::Pop
    }
}
