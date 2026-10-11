use cof_core::storage::SaveLocation;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::page;

/// App-wide settings: where campaigns are saved.
pub struct AppSettings;

impl Screen for AppSettings {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let help = [("←→/enter", "change"), ("esc", "back")];
        let body = page(frame, area, "Settings", &help);
        let current = ctx.config.save_location;

        let mut text = vec![
            Line::from(vec![
                Span::styled("Save location   ", theme::text()),
                Span::styled(format!("◀ {} ▶", current.name()), theme::selected()),
            ]),
            Line::default(),
            Line::styled(description(current), theme::dim()),
            Line::default(),
        ];
        text.push(Line::from(vec![
            Span::styled("Campaigns are in ", theme::dim()),
            Span::styled(ctx.store.dir().display().to_string(), theme::text()),
        ]));
        text.push(Line::styled(
            "Switching doesn't move existing saves; each location keeps its own campaigns.",
            theme::dim(),
        ));
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), body);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Transition::Pop,
            KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => {
                let next = match ctx.config.save_location {
                    SaveLocation::System => SaveLocation::Local,
                    SaveLocation::Local => SaveLocation::System,
                };
                ctx.set_save_location(next);
                if ctx.status.is_none() {
                    ctx.info(format!("Now saving to the {}.", next.name().to_lowercase()));
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }
}

fn description(location: SaveLocation) -> &'static str {
    match location {
        SaveLocation::System => {
            "Saves live in your system's application data folder, so you see the same \
             campaigns wherever you launch the toolbox from. (Default.)"
        }
        SaveLocation::Local => {
            "Saves live in a `saves` folder inside the directory you launch the toolbox \
             from. Handy for keeping a campaign next to your notes or in a synced folder."
        }
    }
}
