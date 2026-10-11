//! Oracle & Table Roller: the solo player's GM (p.84–93), plus dice and Usage Dice.

mod input;
mod usage_dice;
mod view;

use cof_core::dice::{DiceExpr, Die};
use cof_core::oracle::{Disposition, Likelihood};
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::widgets::Menu;

/// The procedures on the Oracle's menu, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Procedure {
    YesNo,
    ActionTheme,
    Surprise,
    Interest,
    Resolution,
    Npc,
    Dice,
    UsageDice,
}

impl Procedure {
    const ALL: [Procedure; 8] = [
        Procedure::YesNo,
        Procedure::ActionTheme,
        Procedure::Surprise,
        Procedure::Interest,
        Procedure::Resolution,
        Procedure::Npc,
        Procedure::Dice,
        Procedure::UsageDice,
    ];

    fn label(self) -> &'static str {
        match self {
            Procedure::YesNo => "Yes/No Oracle",
            Procedure::ActionTheme => "Action & Theme",
            Procedure::Surprise => "Surprise Event",
            Procedure::Interest => "Anything of Interest?",
            Procedure::Resolution => "Random Action Resolution",
            Procedure::Npc => "Generate NPC",
            Procedure::Dice => "Roll Dice",
            Procedure::UsageDice => "Usage Dice",
        }
    }
}

/// The most recent result, shown in full.
struct LastResult {
    title: &'static str,
    text: String,
}

pub struct OracleScreen {
    menu: Menu,
    likelihood: Likelihood,
    disposition: Disposition,
    resolution_modifier: i8,
    /// The free dice roller: how many of which die (no modifier; players add their own).
    dice_count: u8,
    dice_die: Die,
    /// Set while asking "clear the whole journal?".
    confirm_clear: bool,
    last: Option<LastResult>,
}

impl Default for OracleScreen {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            likelihood: Likelihood::default(),
            disposition: Disposition::default(),
            resolution_modifier: 0,
            dice_count: 1,
            dice_die: Die::D20,
            confirm_clear: false,
            last: None,
        }
    }
}

impl OracleScreen {
    fn selected(&self) -> Procedure {
        let index = self.menu.selected(Procedure::ALL.len()).unwrap_or(0);
        Procedure::ALL[index]
    }

    fn dice(&self) -> DiceExpr {
        DiceExpr {
            count: self.dice_count,
            die: self.dice_die,
            modifier: 0,
        }
    }

    /// Shows a result and writes it to the campaign journal (autosaved).
    fn record(&mut self, title: &'static str, text: String, ctx: &mut AppContext) {
        if let Some(campaign) = ctx.campaign.as_mut() {
            campaign.record(title, text.clone());
            ctx.save_campaign();
        }
        self.last = Some(LastResult { title, text });
    }
}

impl Screen for OracleScreen {
    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        view::draw(self, frame, area, ctx);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        input::handle_key(self, key, ctx)
    }
}
