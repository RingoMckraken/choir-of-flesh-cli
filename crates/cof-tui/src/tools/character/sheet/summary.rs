//! The right half of the sheet: everything about the Unbroken at a glance.

use cof_core::character::{Attribute, Character, Lore};
use cof_core::rules::TraumaTier;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use crate::theme;
use crate::widgets::detail_box;

pub(super) fn draw(frame: &mut Frame, area: Rect, c: &Character) {
    let [stats, story] =
        Layout::vertical([Constraint::Percentage(55), Constraint::Min(0)]).areas(area);
    // Three Attributes per line when there's room, two in a narrow terminal.
    let per_line = if stats.width >= 50 { 3 } else { 2 };
    detail_box(frame, stats, "Sheet", stat_lines(c, per_line));
    detail_box(frame, story, "Body & Soul", story_lines(c));
}

fn stat_lines(c: &Character, per_line: usize) -> Vec<Line<'static>> {
    let attr = |a: Attribute| {
        Span::styled(
            format!(
                "{} {:>2} ({:+})   ",
                a.abbr(),
                c.attributes.score(a),
                c.modifier(a)
            ),
            theme::text(),
        )
    };
    let mut lines: Vec<Line> = Attribute::ALL
        .chunks(per_line)
        .map(|row| Line::from(row.iter().map(|&a| attr(a)).collect::<Vec<_>>()))
        .collect();
    lines.push(Line::default());
    let mastery = c.defense_mastery();
    let dr = if mastery > 0 {
        format!("{} (+{mastery} Mastery)", c.defense_rating())
    } else {
        c.defense_rating().to_string()
    };
    lines.push(Line::from(vec![
        label("DR"),
        value(dr),
        label("  Init"),
        value(format!("{:+}", c.initiative())),
        label("  Move"),
        value(c.move_speed().to_string()),
    ]));
    lines.push(Line::from(vec![
        label("Slots"),
        value(format!("{}/{}", c.inventory.slots_used(), c.item_slots())),
        label("  Injuries"),
        value(format!("{} Major / 8 Minor", c.max_major_injuries())),
    ]));
    lines.push(Line::default());
    let names = |v: Vec<&str>| {
        if v.is_empty() {
            "—".to_owned()
        } else {
            v.join(", ")
        }
    };
    lines.push(listed(
        "Feats",
        names(c.feats.iter().map(|f| f.name()).collect()),
    ));
    lines.push(listed(
        "Drawbacks",
        names(c.drawbacks.iter().map(|d| d.name()).collect()),
    ));
    lines.push(listed(
        "Skills",
        names(c.skills.iter().map(|s| s.name()).collect()),
    ));
    lines.push(listed(
        "Proficient",
        names(c.proficiencies.iter().map(|p| p.as_str()).collect()),
    ));
    if !c.gifts.is_empty() || !c.stigmata.is_empty() {
        let titles = |lore: Lore, rolls: &[u8]| {
            names(
                rolls
                    .iter()
                    .map(|&r| lore.entry(r).title.as_str())
                    .collect(),
            )
        };
        lines.push(listed("Gifts", titles(Lore::Gift, &c.gifts)));
        lines.push(listed("Stigmata", titles(Lore::Stigma, &c.stigmata)));
        if c.too_far_gone() {
            lines.push(Line::styled(
                "Too Far Gone: God no longer hears their Prayers.",
                theme::error(),
            ));
        }
    }
    lines
}

fn story_lines(c: &Character) -> Vec<Line<'static>> {
    let b = &c.background;
    let mut lines = vec![
        listed("Sin", b.sin().title.clone()),
        listed("Shard", b.shard().title.clone()),
        listed("Doom", b.doom().title.clone()),
        listed("Former life", b.occupation().title.clone()),
        listed("Trouble", b.occupation().trouble.clone()),
    ];
    if let (Some(rel), Some(entry)) = (&b.relationship, b.relationship()) {
        let with = if rel.with.trim().is_empty() {
            String::new()
        } else {
            format!(" with {}", rel.with.trim())
        };
        lines.push(listed("Bond", format!("{}{with}", entry.title)));
    }
    lines.push(Line::default());
    lines.push(listed("Humanity", c.humanity.effect().to_owned()));
    let tier = c.trauma.tier();
    if tier != TraumaTier::Unscarred {
        lines.push(listed(tier.name(), tier.effect().to_owned()));
        if tier > TraumaTier::Shaken {
            lines.push(Line::styled(
                "(and every tier below it: Trauma effects stack)",
                theme::dim(),
            ));
        }
    }
    if c.anguish.permanent() > 0 {
        lines.push(listed(
            "Anguish",
            format!(
                "{} of it permanent (The Voice Within)",
                c.anguish.permanent()
            ),
        ));
    }
    if c.anguish.next_gain_doubled() {
        lines.push(listed(
            "Anguish",
            "the next gain is doubled (Defied Clarity)".into(),
        ));
    }
    if !c.phobias.is_empty() {
        lines.push(listed("Phobias", c.phobias.join(", ")));
    }
    lines
}

fn label(text: &str) -> Span<'static> {
    Span::styled(format!("{text} "), theme::dim())
}

fn value(text: String) -> Span<'static> {
    Span::styled(text, theme::key())
}

fn listed(name: &str, text: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{name:<11} "), theme::dim()),
        Span::styled(text, theme::text()),
    ])
}
