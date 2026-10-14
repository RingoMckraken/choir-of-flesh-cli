use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::campaign_settings::CampaignSettingsScreen;
use super::coming_soon::ComingSoon;
use super::local_time;
use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::tools::character::CharacterTool;
use crate::tools::oracle::OracleScreen;
use crate::widgets::{Menu, page, render_menu};

/// An entry on the campaign hub: a tool (with the milestone it arrives in) or settings.
enum Entry {
    Oracle,
    Character,
    Tool { name: &'static str, milestone: u8 },
    Settings,
}

const ENTRIES: [Entry; 8] = [
    Entry::Oracle,
    Entry::Character,
    Entry::Tool {
        name: "Encounter Generator",
        milestone: 5,
    },
    Entry::Tool {
        name: "Battle Map Simulator",
        milestone: 6,
    },
    Entry::Tool {
        name: "Endless Travail",
        milestone: 7,
    },
    Entry::Tool {
        name: "Incursion Explorer",
        milestone: 8,
    },
    Entry::Tool {
        name: "Settlement Manager",
        milestone: 9,
    },
    Entry::Settings,
];

/// The open campaign's home: pick a tool.
#[derive(Default)]
pub struct CampaignHub {
    menu: Menu,
}

impl Screen for CampaignHub {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(campaign) = &ctx.campaign else {
            return;
        };
        let help = [("↑↓", "select"), ("enter", "open"), ("esc", "settlements")];
        let body = page(frame, area, &campaign.name, &help);
        let [header, list] =
            Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(body);

        let founded = Line::from(vec![
            Span::styled("Founded ", theme::dim()),
            Span::styled(local_time(campaign.created), theme::text()),
        ]);
        frame.render_widget(Paragraph::new(founded), header);

        let items = ENTRIES
            .iter()
            .map(|entry| match entry {
                Entry::Oracle => Line::styled("Oracle & Table Roller", theme::text()),
                Entry::Character => Line::from(vec![
                    Span::styled(
                        format!("{:<30}", "Character Creator & Sheet"),
                        theme::text(),
                    ),
                    Span::styled(character_status(ctx), theme::dim()),
                ]),
                Entry::Tool { name, milestone } => Line::from(vec![
                    Span::styled(format!("{name:<30}"), theme::text()),
                    Span::styled(format!("coming in Milestone {milestone}"), theme::dim()),
                ]),
                Entry::Settings => Line::styled("Campaign Settings", theme::text()),
            })
            .collect();
        render_menu(frame, list, self.menu, items);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        if self.menu.handle_key(key, ENTRIES.len()) {
            return Transition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                ctx.save_campaign();
                ctx.campaign = None;
                ctx.refresh_campaigns();
                Transition::Pop
            }
            KeyCode::Enter => match self.menu.selected(ENTRIES.len()).map(|i| &ENTRIES[i]) {
                Some(Entry::Oracle) => Transition::Push(Box::new(OracleScreen::default())),
                Some(Entry::Character) => Transition::Push(Box::new(CharacterTool::default())),
                Some(Entry::Tool { name, milestone }) => {
                    Transition::Push(Box::new(ComingSoon::new(name, *milestone)))
                }
                Some(Entry::Settings) => Transition::Push(Box::new(CampaignSettingsScreen)),
                None => Transition::Stay,
            },
            _ => Transition::Stay,
        }
    }
}

/// "Godwin, level 2", or a nudge to raise someone.
fn character_status(ctx: &AppContext) -> String {
    match ctx.character() {
        Some(c) => format!("{}, level {}", c.name, c.level),
        None => "no one defends the settlement".to_owned(),
    }
}
