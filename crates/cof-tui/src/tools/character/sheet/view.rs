use cof_core::character::{Character, xp_to_next};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{Row, SheetScreen, rows, summary};
use crate::context::AppContext;
use crate::theme;
use crate::widgets::{detail_box, page, render_menu, text_lines};

pub(super) fn draw(screen: &SheetScreen, frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let Some(c) = ctx.character() else {
        return;
    };
    let help = [
        ("↑↓", "select"),
        ("←→", "adjust"),
        ("+/-", "±10"),
        ("enter", "do it"),
        ("esc", "back"),
    ];
    let body = page(frame, area, &format!("{} · Unbroken", c.name), &help);
    let [header, _, main] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(body);
    frame.render_widget(Paragraph::new(header_line(c, ctx)), header);

    let [left, right] =
        Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).areas(main);
    let rows = rows(c);
    let [menu_area, result_area] = Layout::vertical([
        Constraint::Length(rows.len() as u16 + 1),
        Constraint::Min(0),
    ])
    .areas(left);
    let items = rows.iter().map(|&r| menu_line(c, r, ctx)).collect();
    render_menu(frame, menu_area, screen.menu, items);
    match &screen.last {
        Some(last) => detail_box(
            frame,
            result_area,
            last.title,
            text_lines(&last.text, theme::text()),
        ),
        None => detail_box(
            frame,
            result_area,
            "",
            vec![Line::styled(
                "Changes are saved, and the important ones are written to the journal.",
                theme::dim(),
            )],
        ),
    }
    summary::draw(frame, right, c);
}

fn header_line(c: &Character, ctx: &AppContext) -> Line<'static> {
    let rumors = ctx.campaign.as_ref().map_or(0, |cp| cp.rumors);
    let mut spans = vec![
        Span::styled(format!("Level {}", c.level), theme::key()),
        Span::styled(
            format!("   XP {}/{}", c.xp, xp_to_next(c.level)),
            theme::text(),
        ),
        Span::styled(format!("   DP {}", c.dp), theme::text()),
        Span::styled(format!("   Rumors {rumors}"), theme::text()),
    ];
    if !c.pending_bonus_skills.is_empty() {
        spans.push(Span::styled(
            "   A bonus Skill is owed (Advance)",
            theme::info(),
        ));
    }
    if c.is_encumbered() {
        spans.push(Span::styled("   ENCUMBERED", theme::error()));
    }
    Line::from(spans)
}

fn menu_line(c: &Character, row: Row, ctx: &AppContext) -> Line<'static> {
    let ready = |used: bool| if used { "used this session" } else { "ready" }.to_owned();
    let value = match row {
        Row::Humanity => Some(format!("◀ {}/{} ▶", c.humanity.current(), c.humanity.max())),
        Row::Anguish => Some(format!("◀ {} ▶", c.anguish.current())),
        Row::Trauma => Some(format!(
            "◀ {} {} ▶",
            c.trauma.current(),
            c.trauma.tier().name()
        )),
        Row::Vigor => Some(format!("◀ {}/{} ▶", c.vigor(), c.max_vigor())),
        Row::Xp => Some(format!("◀ {}/{} ▶", c.xp, xp_to_next(c.level))),
        Row::Rumors => Some(format!(
            "◀ {} ▶",
            ctx.campaign.as_ref().map_or(0, |cp| cp.rumors)
        )),
        Row::Rations => Some(format!("◀ {} ▶", c.inventory.rations)),
        Row::Lucky => Some(ready(c.session.lucky_used)),
        Row::Defiance => Some(ready(c.session.defiance_used)),
        Row::Catharsis => Some("available".into()),
        Row::EndSession => Some(format!("+{} XP", cof_core::character::SESSION_XP)),
        Row::Inventory | Row::Advance | Row::Details | Row::Memorial | Row::EndStory => {
            Some("▸".into())
        }
        Row::Rest => None,
    };
    let mut spans = vec![Span::styled(format!("{:<18}", row.label()), theme::text())];
    if let Some(v) = value {
        spans.push(Span::styled(v, theme::key()));
    }
    Line::from(spans)
}
