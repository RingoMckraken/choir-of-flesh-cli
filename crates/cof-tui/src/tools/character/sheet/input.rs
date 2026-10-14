use cof_core::character::{Attribute, Character, SESSION_XP};
use cof_core::dice::{Die, RollMode};
use cof_core::rules::{Check, Difficulty};
use crossterm::event::{KeyCode, KeyEvent};

use super::{Row, SheetScreen, rows};
use crate::context::AppContext;
use crate::screen::Transition;
use crate::tools::character::advance::AdvanceScreen;
use crate::tools::character::anguish_text;
use crate::tools::character::breakdown::BreakdownScreen;
use crate::tools::character::details::DetailsScreen;
use crate::tools::character::fall::{FallScreen, Preset};
use crate::tools::character::inventory::InventoryScreen;
use crate::tools::character::memorial::MemorialScreen;

pub(super) fn handle_key(
    screen: &mut SheetScreen,
    key: KeyEvent,
    ctx: &mut AppContext,
) -> Transition {
    let Some(character) = ctx.character() else {
        return Transition::Pop;
    };
    if screen.confirm_end_session {
        screen.confirm_end_session = false;
        if key.code == KeyCode::Char('y') {
            end_session(screen, ctx);
        }
        return Transition::Stay;
    }
    let len = rows(character).len();
    if screen.menu.handle_key(key, len) {
        return Transition::Stay;
    }
    let Some(row) = screen.selected_row(ctx) else {
        return Transition::Stay;
    };
    let transition = match key.code {
        KeyCode::Esc | KeyCode::Char('q') => return Transition::Pop,
        KeyCode::Left => adjust(screen, row, -1, ctx),
        KeyCode::Right => adjust(screen, row, 1, ctx),
        KeyCode::Char('-' | '_') => adjust(screen, row, -10, ctx),
        KeyCode::Char('+' | '=') => adjust(screen, row, 10, ctx),
        KeyCode::Enter | KeyCode::Char(' ') => activate(screen, row, ctx),
        _ => Transition::Stay,
    };
    ctx.save_campaign();
    transition
}

fn with_character<T>(ctx: &mut AppContext, f: impl FnOnce(&mut Character) -> T) -> Option<T> {
    ctx.character_mut().map(f)
}

/// ←/→ (and -/+ for bigger steps) on a tracker row.
fn adjust(screen: &mut SheetScreen, row: Row, step: i16, ctx: &mut AppContext) -> Transition {
    let amount = step.unsigned_abs();
    let up = step > 0;
    match row {
        Row::Humanity => {
            let (text, lost) = with_character(ctx, |c| {
                if up {
                    c.humanity.regain(amount as u8);
                } else {
                    c.humanity.lose(amount as u8);
                }
                let h = c.humanity;
                (
                    format!("Humanity → {}/{}.", h.current(), h.max()),
                    c.is_lost(),
                )
            })
            .unwrap_or_default();
            screen.report(ctx, "Humanity", text);
            if lost {
                return Transition::Push(Box::new(FallScreen::new(Preset::Lost)));
            }
        }
        Row::Anguish if up => {
            let Some(gain) = with_character(ctx, |c| c.gain_anguish(1)) else {
                return Transition::Stay;
            };
            screen.report(ctx, "Anguish", anguish_text::describe(&gain));
            if let Some(b) = gain.check.and_then(|c| c.breakdown) {
                return Transition::Push(Box::new(BreakdownScreen::new(b)));
            }
        }
        Row::Anguish => {
            let text = with_character(ctx, |c| {
                c.anguish.lose(1);
                format!("Anguish → {}.", c.anguish.current())
            });
            screen.report(ctx, "Anguish", text.unwrap_or_default());
        }
        Row::Trauma => {
            let (text, dead) = with_character(ctx, |c| {
                if up {
                    let gain = c.gain_trauma(1);
                    let tier = if gain.tier_changed() {
                        format!(" Now {}.", gain.to.name())
                    } else {
                        String::new()
                    };
                    (
                        format!("Trauma +1 → {}.{tier}", gain.total),
                        c.is_dead_of_trauma(),
                    )
                } else {
                    c.trauma.heal(1);
                    (format!("Trauma → {}.", c.trauma.current()), false)
                }
            })
            .unwrap_or_default();
            screen.report(ctx, "Trauma", text);
            if dead {
                return Transition::Push(Box::new(FallScreen::new(Preset::Trauma)));
            }
        }
        Row::Vigor => {
            with_character(ctx, |c| {
                if up {
                    c.vigor_spent = c.vigor_spent.saturating_sub(1);
                } else {
                    c.spend_vigor();
                }
            });
        }
        Row::Xp if up => {
            let levels = with_character(ctx, |c| (c.gain_xp(amount), c.dp)).unwrap_or_default();
            report_levels(screen, ctx, &levels.0, levels.1);
        }
        Row::Xp => {
            with_character(ctx, |c| c.lose_xp(amount));
        }
        Row::Rumors => {
            if let Some(campaign) = ctx.campaign.as_mut() {
                campaign.rumors = campaign.rumors.saturating_add_signed(step);
                if up && campaign.rumors >= 10 {
                    ctx.info("10 Rumors: enough to set out on an Incursion (p.110).");
                }
            }
        }
        Row::Rations => {
            with_character(ctx, |c| {
                c.inventory.rations = c.inventory.rations.saturating_add_signed(step);
            });
        }
        _ => {}
    }
    Transition::Stay
}

fn activate(screen: &mut SheetScreen, row: Row, ctx: &mut AppContext) -> Transition {
    match row {
        Row::Rest => {
            with_character(ctx, Character::rest);
            screen.report(ctx, "Rest", "Six hours of rest: Vigor restored.".into());
        }
        Row::EndSession => {
            screen.confirm_end_session = true;
            ctx.info(format!(
                "End the session? +{SESSION_XP} XP and the once-per-session flags reset. (y/n)"
            ));
        }
        Row::Lucky => {
            let used = with_character(ctx, Character::use_lucky).unwrap_or(false);
            if used {
                screen.report(
                    ctx,
                    "Lucky",
                    "Lucky: reroll a check. Used for this session.".into(),
                );
            } else {
                ctx.error("Lucky is already spent this session.");
            }
        }
        Row::Defiance => {
            let text = with_character(ctx, |c| {
                c.act_of_defiance().then(|| {
                    format!(
                        "An Act of Defiance: +1 Humanity → {}.",
                        c.humanity.current()
                    )
                })
            })
            .flatten();
            match text {
                Some(text) => screen.report(ctx, "Act of Defiance", text),
                None => ctx.error("An Act of Defiance already counted this session."),
            }
        }
        Row::Catharsis => catharsis(screen, ctx),
        Row::Inventory => return Transition::Push(Box::new(InventoryScreen::default())),
        Row::Advance => return Transition::Push(Box::new(AdvanceScreen::default())),
        Row::Details => return Transition::Push(Box::new(DetailsScreen::default())),
        Row::Memorial => return Transition::Push(Box::new(MemorialScreen::default())),
        Row::EndStory => return Transition::Push(Box::new(FallScreen::new(Preset::None))),
        Row::Anguish => return adjust(screen, row, 1, ctx),
        _ => {}
    }
    Transition::Stay
}

/// Catharsis (p.48): once per Breakdown, in a moment of safety, an Average WIL check; on a
/// success, lose D4 Anguish. Battered Trauma's permanent Fatigue gives Disadvantage.
fn catharsis(screen: &mut SheetScreen, ctx: &mut AppContext) {
    let table = ctx
        .campaign
        .as_ref()
        .map(|c| c.settings.tn_table)
        .unwrap_or_default();
    let text = with_character(ctx, |c| {
        let mode = if c.trauma.permanently_fatigued() {
            RollMode::Disadvantage
        } else {
            RollMode::Normal
        };
        let outcome = Check::dynamic(table, Difficulty::Average, c.modifier(Attribute::Wil))
            .with_mode(mode)
            .roll();
        c.catharsis_available = false;
        let roll = format!(
            "Average WIL check: D20 = {}, total {} vs TN {}",
            outcome.natural, outcome.total, outcome.tn
        );
        if outcome.result.is_success() {
            let relief = Die::D4.roll();
            c.anguish.lose(relief);
            format!(
                "{roll}: success. Catharsis: -{relief} Anguish → {}.",
                c.anguish.current()
            )
        } else {
            format!("{roll}: failure. The moment passes.")
        }
    });
    screen.report(ctx, "Catharsis", text.unwrap_or_default());
}

fn end_session(screen: &mut SheetScreen, ctx: &mut AppContext) {
    let Some((levels, dp)) = with_character(ctx, |c| (c.end_session(), c.dp)) else {
        return;
    };
    screen.report(
        ctx,
        "End Session",
        format!("Session over: +{SESSION_XP} XP. Lucky and Acts of Defiance are ready again."),
    );
    report_levels(screen, ctx, &levels, dp);
    ctx.save_campaign();
}

fn report_levels(screen: &mut SheetScreen, ctx: &mut AppContext, levels: &[u8], dp: u16) {
    if let Some(&level) = levels.last() {
        screen.report(
            ctx,
            "Level Up",
            format!("Level {level}! Development Points: {dp}. Spend them under Advance."),
        );
    }
}
