use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Wrap};

use crate::theme;

/// A rounded, dimly bordered box of wrapped text: descriptions, summaries, roll results.
pub fn detail_box(frame: &mut Frame, area: Rect, title: &str, lines: Vec<Line<'_>>) {
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::dim())
        .padding(Padding::horizontal(1));
    if !title.is_empty() {
        block = block.title(Line::styled(format!(" {title} "), theme::title()));
    }
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

/// Text as wrapped lines in one style.
pub fn text_lines(text: &str, style: ratatui::style::Style) -> Vec<Line<'static>> {
    text.lines()
        .map(|l| Line::styled(l.to_owned(), style))
        .collect()
}
