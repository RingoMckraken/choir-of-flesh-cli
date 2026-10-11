use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{List, ListState};

use crate::theme;

/// The selection position in a vertical list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Menu {
    selected: usize,
}

impl Menu {
    /// The selected index, kept within a list of `len` items.
    pub fn selected(&self, len: usize) -> Option<usize> {
        (len > 0).then(|| self.selected.min(len - 1))
    }

    /// Moves the selection for arrow keys (and j/k). Returns `true` if the key was used.
    pub fn handle_key(&mut self, key: KeyEvent, len: usize) -> bool {
        let Some(current) = self.selected(len) else {
            return false;
        };
        self.selected = match key.code {
            KeyCode::Up | KeyCode::Char('k') => current.checked_sub(1).unwrap_or(len - 1),
            KeyCode::Down | KeyCode::Char('j') => (current + 1) % len,
            KeyCode::Home => 0,
            KeyCode::End => len - 1,
            _ => return false,
        };
        true
    }
}

/// Renders `items` as a list with the menu's selection highlighted.
pub fn render_menu<'a>(frame: &mut Frame, area: Rect, menu: Menu, items: Vec<Line<'a>>) {
    let mut state = ListState::default().with_selected(menu.selected(items.len()));
    let list = List::new(items)
        .style(theme::text())
        .highlight_style(theme::selected())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(menu: &mut Menu, code: KeyCode, len: usize) {
        menu.handle_key(KeyEvent::from(code), len);
    }

    #[test]
    fn arrows_wrap_around() {
        let mut menu = Menu::default();
        press(&mut menu, KeyCode::Up, 3);
        assert_eq!(menu.selected(3), Some(2));
        press(&mut menu, KeyCode::Down, 3);
        assert_eq!(menu.selected(3), Some(0));
    }

    #[test]
    fn selection_clamps_when_the_list_shrinks() {
        let mut menu = Menu::default();
        press(&mut menu, KeyCode::End, 5);
        assert_eq!(menu.selected(2), Some(1));
        assert_eq!(menu.selected(0), None);
    }

    #[test]
    fn other_keys_are_not_used() {
        let mut menu = Menu::default();
        assert!(!menu.handle_key(KeyEvent::from(KeyCode::Enter), 3));
    }
}
