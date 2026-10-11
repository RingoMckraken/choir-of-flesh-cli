use cof_core::rules::TnTable;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::page;

/// Rules options for the open campaign. Changes are saved immediately.
pub struct CampaignSettingsScreen;

impl Screen for CampaignSettingsScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(campaign) = &ctx.campaign else {
            return;
        };
        let help = [("←→/enter", "change"), ("esc", "back")];
        let body = page(frame, area, "Campaign Settings", &help);
        let current = campaign.settings.tn_table;
        let text = vec![
            Line::from(vec![
                Span::styled("Dynamic Target Numbers   ", theme::text()),
                Span::styled(format!("◀ {} ▶", tn_name(current)), theme::selected()),
            ]),
            Line::default(),
            Line::styled(tn_description(current), theme::dim()),
        ];
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), body);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Transition::Pop,
            KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Char(' ') => {
                let Some(campaign) = ctx.campaign.as_mut() else {
                    return Transition::Stay;
                };
                let next = match campaign.settings.tn_table {
                    TnTable::Standard => TnTable::Scaling,
                    TnTable::Scaling => TnTable::Standard,
                };
                campaign.settings.tn_table = next;
                ctx.save_campaign();
                if ctx.status.is_none() {
                    ctx.info(format!("Saved: {} table.", tn_name(next)));
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }
}

fn tn_name(table: TnTable) -> &'static str {
    match table {
        TnTable::Standard => "Standard",
        TnTable::Scaling => "Scaling",
    }
}

fn tn_description(table: TnTable) -> &'static str {
    match table {
        TnTable::Standard => {
            "The default table (p.43). Target Numbers rise one-for-one with your modifiers, \
             so challenges keep pace with you and only Mastery and Burden tip the odds."
        }
        TnTable::Scaling => {
            "Appendix II (p.230). Threats still scale with your power, but more slowly, so \
             higher Attributes make a noticeable difference."
        }
    }
}
