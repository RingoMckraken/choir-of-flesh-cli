use cof_core::storage::CampaignSummary;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::app_settings::AppSettings;
use super::campaign_hub::CampaignHub;
use super::local_time;
use super::new_campaign::NewCampaign;
use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, page, render_menu};

/// The list of campaigns (settlements), with new / open / delete.
#[derive(Default)]
pub struct MainMenu {
    menu: Menu,
    /// Set while asking "delete this campaign?".
    confirm_delete: bool,
}

impl MainMenu {
    fn selected<'c>(&self, ctx: &'c AppContext) -> Option<&'c CampaignSummary> {
        let campaigns = &ctx.campaigns.campaigns;
        self.menu.selected(campaigns.len()).map(|i| &campaigns[i])
    }

    fn open(&self, ctx: &mut AppContext) -> Transition {
        let Some(id) = self.selected(ctx).map(|c| c.id.clone()) else {
            return Transition::Stay;
        };
        match ctx.store.load(&id) {
            Ok(campaign) => {
                ctx.campaign = Some(campaign);
                Transition::Push(Box::new(CampaignHub::default()))
            }
            Err(e) => {
                ctx.error(format!("Couldn't open: {e}"));
                Transition::Stay
            }
        }
    }

    fn confirm(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        self.confirm_delete = false;
        let Some(summary) = self.selected(ctx).cloned() else {
            return Transition::Stay;
        };
        if key.code != KeyCode::Char('y') {
            ctx.info(format!("{} still stands.", summary.name));
            return Transition::Stay;
        }
        match ctx.store.delete(&summary.id) {
            Ok(()) => {
                ctx.refresh_campaigns();
                ctx.info(format!("{} is gone, swallowed by the Flesh.", summary.name));
            }
            Err(e) => ctx.error(format!("Couldn't delete: {e}")),
        }
        Transition::Stay
    }
}

impl Screen for MainMenu {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let help: &[(&str, &str)] = if self.confirm_delete {
            &[("y", "delete forever"), ("any other key", "keep it")]
        } else {
            &[
                ("↑↓", "select"),
                ("enter", "open"),
                ("n", "new"),
                ("d", "delete"),
                ("s", "settings"),
                ("q", "quit"),
            ]
        };
        let body = page(frame, area, "Choir of Flesh · Settlements", help);
        let [list_area, footer] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(body);

        let campaigns = &ctx.campaigns.campaigns;
        if campaigns.is_empty() {
            let text = vec![
                Line::styled("No settlements stand yet.", theme::text()),
                Line::default(),
                Line::from(vec![
                    Span::styled("Press ", theme::dim()),
                    Span::styled("n", theme::key()),
                    Span::styled(" to found one.", theme::dim()),
                ]),
            ];
            frame.render_widget(Paragraph::new(text), list_area);
        } else {
            let items = campaigns
                .iter()
                .map(|c| {
                    Line::from(vec![
                        Span::styled(format!("{:<32}", c.name), theme::text()),
                        Span::styled(
                            format!("last played {}", local_time(c.updated)),
                            theme::dim(),
                        ),
                    ])
                })
                .collect();
            render_menu(frame, list_area, self.menu, items);
        }

        let footer_text = match self.selected(ctx) {
            Some(c) if self.confirm_delete => Line::styled(
                format!(
                    "Delete {} and everything in it? This can't be undone.",
                    c.name
                ),
                theme::error(),
            ),
            _ => Line::styled(
                format!("Saves: {}", ctx.store.dir().display()),
                theme::dim(),
            ),
        };
        frame.render_widget(
            Paragraph::new(footer_text).wrap(Wrap { trim: true }),
            footer,
        );
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.confirm_delete {
            return self.confirm(key, ctx);
        }
        if self.menu.handle_key(key, ctx.campaigns.campaigns.len()) {
            return Transition::Stay;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Transition::Quit,
            KeyCode::Char('n') => Transition::Push(Box::new(NewCampaign::default())),
            KeyCode::Char('s') => Transition::Push(Box::new(AppSettings)),
            KeyCode::Char('d') if self.selected(ctx).is_some() => {
                self.confirm_delete = true;
                Transition::Stay
            }
            KeyCode::Enter => self.open(ctx),
            _ => Transition::Stay,
        }
    }
}
