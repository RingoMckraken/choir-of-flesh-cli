use cof_core::dice::{DiceExpr, Die};
use cof_core::oracle;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::usage_dice::UsageDiceScreen;
use super::{OracleScreen, Procedure};
use crate::context::AppContext;
use crate::screen::Transition;

pub(super) fn handle_key(
    screen: &mut OracleScreen,
    key: KeyEvent,
    ctx: &mut AppContext,
) -> Transition {
    if screen.confirm_clear {
        return confirm_clear(screen, key, ctx);
    }
    if is_clear_journal(key) {
        return ask_clear(screen, ctx);
    }
    if screen.menu.handle_key(key, Procedure::ALL.len()) {
        return Transition::Stay;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Transition::Pop,
        KeyCode::Left => adjust(screen, -1),
        KeyCode::Right => adjust(screen, 1),
        KeyCode::Char('+' | '=') => change_count(screen, 1),
        KeyCode::Char('-' | '_') => change_count(screen, -1),
        KeyCode::Enter | KeyCode::Char(' ') => run(screen, ctx),
        _ => Transition::Stay,
    }
}

/// Shift+C, which arrives as an uppercase 'C' (with or without the SHIFT flag, depending on
/// the terminal). Lowercase 'c' and Ctrl+C don't count.
fn is_clear_journal(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('C') && !key.modifiers.contains(KeyModifiers::CONTROL)
}

fn ask_clear(screen: &mut OracleScreen, ctx: &mut AppContext) -> Transition {
    let entries = ctx.campaign.as_ref().map_or(0, |c| c.journal.len());
    if entries == 0 {
        ctx.info("The journal is already empty.");
    } else {
        screen.confirm_clear = true;
        let plural = if entries == 1 { "entry" } else { "entries" };
        ctx.error(format!(
            "Clear all {entries} journal {plural}? This can't be undone. Press y to confirm."
        ));
    }
    Transition::Stay
}

fn confirm_clear(screen: &mut OracleScreen, key: KeyEvent, ctx: &mut AppContext) -> Transition {
    screen.confirm_clear = false;
    if key.code != KeyCode::Char('y') {
        ctx.info("The journal is untouched.");
        return Transition::Stay;
    }
    if let Some(campaign) = ctx.campaign.as_mut() {
        campaign.journal.clear();
        ctx.save_campaign();
        if ctx.status.is_none() {
            ctx.info("The journal is wiped clean.");
        }
    }
    Transition::Stay
}

/// ←/→ change the selected procedure's option, if it has one.
fn adjust(screen: &mut OracleScreen, step: i8) -> Transition {
    match screen.selected() {
        Procedure::YesNo => screen.likelihood = screen.likelihood.shifted(step),
        Procedure::Resolution => {
            screen.resolution_modifier = (screen.resolution_modifier + step).clamp(-5, 5);
        }
        Procedure::Npc => screen.disposition = screen.disposition.cycled(step),
        Procedure::Dice => {
            let len = Die::ALL.len() as i16;
            let index = Die::ALL
                .iter()
                .position(|&d| d == screen.dice_die)
                .unwrap_or(0) as i16;
            screen.dice_die = Die::ALL[(index + i16::from(step)).rem_euclid(len) as usize];
        }
        _ => {}
    }
    Transition::Stay
}

/// +/- change how many dice the free roller throws (only on the Roll Dice row).
fn change_count(screen: &mut OracleScreen, step: i8) -> Transition {
    if screen.selected() == Procedure::Dice {
        let count = i16::from(screen.dice_count) + i16::from(step);
        screen.dice_count = count.clamp(1, i16::from(DiceExpr::MAX_DICE)) as u8;
    }
    Transition::Stay
}

fn run(screen: &mut OracleScreen, ctx: &mut AppContext) -> Transition {
    let procedure = screen.selected();
    let text = match procedure {
        Procedure::YesNo => oracle::ask(screen.likelihood).to_string(),
        Procedure::ActionTheme => oracle::action_theme().to_string(),
        Procedure::Surprise => oracle::surprise_event().to_string(),
        Procedure::Interest => oracle::anything_of_interest().to_string(),
        Procedure::Resolution => oracle::resolve_action(screen.resolution_modifier).to_string(),
        Procedure::Npc => oracle::generate_npc(screen.disposition).to_string(),
        Procedure::Dice => screen.dice().roll().to_string(),
        Procedure::UsageDice => return Transition::Push(Box::new(UsageDiceScreen::default())),
    };
    screen.record(procedure.label(), text, ctx);
    Transition::Stay
}
