use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Wrap};

use super::{OracleScreen, Procedure};
use crate::context::AppContext;
use crate::theme;
use crate::widgets::{page, render_menu};

pub(super) fn draw(screen: &OracleScreen, frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let help: &[(&str, &str)] = if screen.confirm_clear {
        &[("y", "clear journal"), ("any other key", "keep it")]
    } else if screen.selected() == Procedure::Dice {
        &[
            ("↑↓", "select"),
            ("←→", "die"),
            ("+/-", "how many"),
            ("enter", "roll"),
            ("shift+c", "clear journal"),
            ("esc", "back"),
        ]
    } else {
        &[
            ("↑↓", "select"),
            ("←→", "option"),
            ("enter", "roll"),
            ("shift+c", "clear journal"),
            ("esc", "back"),
        ]
    };
    let body = page(frame, area, "Oracle & Table Roller", help);
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)]).areas(body);
    let [menu_area, result_area] = Layout::vertical([
        Constraint::Length(Procedure::ALL.len() as u16 + 1),
        Constraint::Min(0),
    ])
    .areas(left);

    let items = Procedure::ALL
        .iter()
        .map(|&p| menu_line(screen, p))
        .collect();
    render_menu(frame, menu_area, screen.menu, items);
    draw_result(screen, frame, result_area);
    draw_journal(frame, right, ctx);
}

fn menu_line(screen: &OracleScreen, procedure: Procedure) -> Line<'static> {
    let option = match procedure {
        Procedure::YesNo => Some(format!(
            "◀ {} {}% ▶",
            screen.likelihood.name(),
            screen.likelihood.chance()
        )),
        Procedure::Resolution => Some(format!("◀ {:+} ▶", screen.resolution_modifier)),
        Procedure::Npc => Some(format!("◀ {} ▶", screen.disposition.name())),
        Procedure::Dice => Some(format!(
            "◀ {} × D{} ▶",
            screen.dice_count,
            screen.dice_die.sides()
        )),
        Procedure::UsageDice => Some("▸".to_owned()),
        _ => None,
    };
    let mut spans = vec![Span::styled(
        format!("{:<25}", procedure.label()),
        theme::text(),
    )];
    if let Some(option) = option {
        spans.push(Span::styled(option, theme::key()));
    }
    Line::from(spans)
}

fn draw_result(screen: &OracleScreen, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::dim())
        .padding(Padding::horizontal(1));
    let lines = match &screen.last {
        Some(last) => {
            let mut lines = vec![Line::styled(last.title, theme::title())];
            lines.extend(
                last.text
                    .lines()
                    .map(|l| Line::styled(l.to_owned(), theme::text())),
            );
            lines
        }
        None => vec![Line::styled(
            "Ask the Oracle. Results are written to the journal.",
            theme::dim(),
        )],
    };
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_journal(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::dim())
        .padding(Padding::horizontal(1))
        .title(Line::styled(" Journal ", theme::title()));
    let entries = ctx
        .campaign
        .as_ref()
        .map(|c| c.journal.as_slice())
        .unwrap_or_default();
    let mut lines = Vec::new();
    for entry in entries.iter().rev().take(50) {
        let time = entry.at.with_timezone(&chrono::Local).format("%H:%M");
        lines.push(Line::from(vec![
            Span::styled(format!("{time} "), theme::dim()),
            Span::styled(entry.source.clone(), theme::key()),
        ]));
        lines.extend(
            entry
                .text
                .lines()
                .map(|l| Line::styled(format!("  {l}"), theme::text())),
        );
    }
    if lines.is_empty() {
        lines.push(Line::styled("Nothing written yet.", theme::dim()));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}
