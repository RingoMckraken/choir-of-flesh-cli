use cof_core::character::creation::Step;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::{Confirm, CreationScreen};
use super::{attributes, background, feats, gear, identity, proficiencies, review, skills};
use crate::context::AppContext;
use crate::theme;
use crate::widgets::page;

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let mut help: Vec<(&str, &str)> = match screen.confirm {
        Some(_) => vec![("y", "yes"), ("any other key", "no")],
        None => step_help(screen.step).to_vec(),
    };
    if screen.confirm.is_none() {
        help.extend([("tab", "next"), ("shift+tab", "back"), ("esc", "cancel")]);
    }
    let title = match &screen.successor_of {
        Some(name) => format!("Raise a Successor to {name}"),
        None => "Raise an Unbroken".to_owned(),
    };
    let body = page(frame, area, &title, &help);
    let [stepper, _, main, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(body);
    frame.render_widget(Paragraph::new(stepper_line(screen)), stepper);

    match screen.step {
        Step::Attributes => attributes::draw(screen, frame, main),
        Step::Proficiencies => proficiencies::draw(screen, frame, main),
        Step::Feats => feats::draw(screen, frame, main),
        Step::Skills => skills::draw(screen, frame, main),
        Step::Background => background::draw(screen, frame, main),
        Step::Identity => identity::draw(screen, frame, main),
        Step::Gear => gear::draw(screen, frame, main),
        Step::Review => review::draw(screen, frame, main, ctx),
    }
    frame.render_widget(Paragraph::new(footer_line(screen)), footer);
}

fn step_help(step: Step) -> &'static [(&'static str, &'static str)] {
    match step {
        Step::Attributes => &[("↑↓", "attribute"), ("←→", "swap roll")],
        Step::Proficiencies => &[("↑↓", "select"), ("←→", "choose")],
        Step::Feats => &[
            ("↑↓", "select"),
            ("enter", "take/drop"),
            ("r", "roll drawback"),
        ],
        Step::Skills => &[
            ("↑↓", "select"),
            ("enter", "pick/drop"),
            ("b", "bonus skill"),
            ("g", "gift"),
        ],
        Step::Background => &[("↑↓", "select"), ("←→", "choose"), ("r", "roll")],
        Step::Identity => &[
            ("↑↓", "select"),
            ("type", "name"),
            ("←→", "choose"),
            ("enter", "roll"),
        ],
        Step::Gear => &[("↑↓", "select"), ("←→", "weapon"), ("+/-", "buy/return")],
        Step::Review => &[("←→", "heirloom"), ("enter", "raise them")],
    }
}

/// "Attributes ✓ › Proficiencies ✓ › Feats …", the current step highlighted.
fn stepper_line(screen: &CreationScreen) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, &step) in Step::ALL.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" › ", theme::dim()));
        }
        let done = step != Step::Review && screen.draft.problem(step).is_none();
        let label = if done {
            format!("{} ✓", short_name(step))
        } else {
            short_name(step).to_owned()
        };
        let style = if step == screen.step {
            theme::selected()
        } else if done {
            theme::info()
        } else {
            theme::dim()
        };
        spans.push(Span::styled(label, style));
    }
    Line::from(spans)
}

/// One-word step names, so the tracker fits an 80-column terminal.
fn short_name(step: Step) -> &'static str {
    match step {
        Step::Attributes => "Attributes",
        Step::Proficiencies => "Proficiencies",
        Step::Feats => "Feats",
        Step::Skills => "Skills",
        Step::Background => "Background",
        Step::Identity => "Name",
        Step::Gear => "Gear",
        Step::Review => "Review",
    }
}

/// The pending question, the last roll, or what's still missing in this step.
fn footer_line(screen: &CreationScreen) -> Line<'static> {
    if let Some(question) = screen.confirm {
        let text = match question {
            Confirm::Cancel => "Abandon this Unbroken? Nothing is saved. (y/n)",
            Confirm::Gift => {
                "Trade a Skill for a rolled Gift of the Unmade? It costs D4 Humanity and 1 \
                 Anguish, and can't be undone. (y/n)"
            }
        };
        return Line::styled(text, theme::error());
    }
    if let Some(note) = &screen.note {
        return Line::styled(note.clone(), theme::info());
    }
    match screen.draft.problem(screen.step) {
        Some(problem) => Line::styled(format!("To do: {problem}."), theme::dim()),
        None if screen.step == Step::Review => {
            Line::styled("Ready. Press enter to raise them.", theme::info())
        }
        None => Line::styled("Done. Press tab for the next step.", theme::info()),
    }
}
