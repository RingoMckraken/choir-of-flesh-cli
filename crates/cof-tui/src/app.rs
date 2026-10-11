use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;

use crate::context::{AppContext, Status};
use crate::screen::{Screen, Transition};
use crate::screens::MainMenu;
use crate::theme;

/// The main loop: draw the top screen, wait for a key, apply the screen's transition.
pub struct App {
    ctx: AppContext,
    stack: Vec<Box<dyn Screen>>,
}

impl App {
    pub fn new(ctx: AppContext) -> Self {
        Self {
            ctx,
            stack: vec![Box::new(MainMenu::default())],
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while let Some(top) = self.stack.last() {
            terminal.draw(|frame| {
                let [body, status] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)])
                    .areas(frame.area());
                top.draw(frame, body, &self.ctx);
                frame.render_widget(status_line(self.ctx.status.as_ref()), status);
            })?;

            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }
            self.ctx.status = None;
            let screen = self.stack.last_mut().expect("loop runs only with a screen");
            match screen.handle_key(key, &mut self.ctx) {
                Transition::Stay => {}
                Transition::Push(next) => self.stack.push(next),
                Transition::Replace(next) => {
                    self.stack.pop();
                    self.stack.push(next);
                }
                Transition::Pop => {
                    self.stack.pop();
                }
                Transition::Quit => break,
            }
        }
        // Whatever happens, leave the open campaign saved.
        self.ctx.save_campaign();
        Ok(())
    }
}

fn status_line(status: Option<&Status>) -> Line<'_> {
    match status {
        Some(Status::Info(text)) => Line::styled(format!(" {text}"), theme::info()),
        Some(Status::Error(text)) => Line::styled(format!(" {text}"), theme::error()),
        None => Line::default(),
    }
}
