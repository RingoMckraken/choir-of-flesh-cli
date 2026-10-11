use cof_core::settlement;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::campaign_hub::CampaignHub;
use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{TextInput, centered, page};

const MAX_NAME: usize = 40;

/// Founding a new settlement, which starts a new campaign.
pub struct NewCampaign {
    name: TextInput,
}

impl Default for NewCampaign {
    fn default() -> Self {
        Self {
            name: TextInput::new(MAX_NAME),
        }
    }
}

impl Screen for NewCampaign {
    fn draw(&self, frame: &mut Frame, area: Rect, _ctx: &AppContext) {
        let help = [
            ("tab", "roll a name"),
            ("enter", "found it"),
            ("esc", "cancel"),
        ];
        let body = page(frame, centered(area, 64, 11), "Found a Settlement", &help);
        let text = vec![
            Line::styled(
                "Unbroken come and go. The settlement endures. Name it:",
                theme::dim(),
            ),
            Line::default(),
            self.name.line(),
            Line::default(),
            Line::styled(
                "Tab rolls on the Settlement Name Generator (p.36).",
                theme::dim(),
            ),
        ];
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), body);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.name.handle_key(key) {
            return Transition::Stay;
        }
        match key.code {
            KeyCode::Esc => Transition::Pop,
            KeyCode::Tab => {
                self.name.set(settlement::roll_name());
                Transition::Stay
            }
            KeyCode::Enter => match ctx.store.create(self.name.value()) {
                Ok(campaign) => {
                    ctx.info(format!("{} is founded.", campaign.name));
                    ctx.campaign = Some(campaign);
                    ctx.refresh_campaigns();
                    Transition::Replace(Box::new(CampaignHub::default()))
                }
                Err(e) => {
                    ctx.error(format!("Couldn't found it: {e}"));
                    Transition::Stay
                }
            },
            _ => Transition::Stay,
        }
    }
}
