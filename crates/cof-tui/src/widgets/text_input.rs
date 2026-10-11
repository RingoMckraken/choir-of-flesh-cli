use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::text::{Line, Span};

use crate::theme;

/// A single-line text field (typing appends, backspace deletes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    value: String,
    max_chars: usize,
}

impl TextInput {
    pub fn new(max_chars: usize) -> Self {
        Self {
            value: String::new(),
            max_chars,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set(&mut self, value: impl Into<String>) {
        self.value = value.into().chars().take(self.max_chars).collect();
    }

    /// Applies a typing key. Returns `true` if the key was used.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && self.value.chars().count() < self.max_chars =>
            {
                self.value.push(c);
                true
            }
            KeyCode::Backspace => self.value.pop().is_some(),
            _ => false,
        }
    }

    /// The field with a block cursor at the end.
    pub fn line(&self) -> Line<'_> {
        Line::from(vec![
            Span::styled(self.value.as_str(), theme::text()),
            Span::styled("█", theme::dim()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_str(input: &mut TextInput, text: &str) {
        for c in text.chars() {
            input.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
    }

    #[test]
    fn typing_and_backspace() {
        let mut input = TextInput::new(20);
        type_str(&mut input, "Grey Fordd");
        input.handle_key(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(input.value(), "Grey Ford");
    }

    #[test]
    fn respects_max_length() {
        let mut input = TextInput::new(4);
        type_str(&mut input, "Lantern");
        assert_eq!(input.value(), "Lant");
        input.set("Hope's Watch");
        assert_eq!(input.value(), "Hope");
    }

    #[test]
    fn control_chords_are_not_typed() {
        let mut input = TextInput::new(10);
        let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
        assert!(!input.handle_key(ctrl_r));
        assert_eq!(input.value(), "");
    }
}
