use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType};

use crate::theme;

/// Draws the standard page frame (a titled border with a key-help line along the bottom)
/// and returns the area inside it for the screen's content.
///
/// `help` is a list of `(key, action)` pairs, e.g. `[("enter", "open"), ("esc", "back")]`.
pub fn page(frame: &mut Frame, area: Rect, title: &str, help: &[(&str, &str)]) -> Rect {
    let mut keys = vec![Span::raw(" ")];
    for (key, action) in help {
        keys.push(Span::styled(*key, theme::key()));
        keys.push(Span::styled(format!(" {action}   "), theme::dim()));
    }
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::border())
        .title(Line::styled(format!(" {title} "), theme::title()))
        .title_bottom(Line::from(keys));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner.inner(ratatui::layout::Margin::new(2, 1))
}

/// A `width` × `height` rectangle centred in `area` (shrunk to fit if needed).
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height.min(area.height))])
        .flex(Flex::Center)
        .areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width.min(area.width))])
        .flex(Flex::Center)
        .areas(row);
    cell
}
