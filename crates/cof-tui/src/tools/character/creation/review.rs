use cof_core::character::creation::{Created, Draft, Step};
use cof_core::character::{Attribute, Lore, occupation};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::CreationScreen;
use crate::context::AppContext;
use crate::screen::Transition;
use crate::theme;
use crate::tools::character::anguish_text;
use crate::tools::character::breakdown::BreakdownScreen;
use crate::widgets::detail_box;

/// The fallen's items a successor may take one of, if the Heirloom rule is on.
fn heirloom_options(ctx: &AppContext) -> Vec<String> {
    let Some(campaign) = &ctx.campaign else {
        return Vec::new();
    };
    if !campaign.settings.heirloom {
        return Vec::new();
    }
    campaign
        .roster
        .heirloom_options()
        .iter()
        .map(|i| i.name.clone())
        .collect()
}

pub(super) fn key(screen: &mut CreationScreen, key: KeyEvent, ctx: &mut AppContext) -> Transition {
    match key.code {
        KeyCode::Left | KeyCode::Right => {
            let count = heirloom_options(ctx).len();
            if count > 0 {
                // Cycles none → 0 … count-1 → none.
                let positions = count as i32 + 1;
                let current = screen.heirloom.map_or(0, |i| i as i32 + 1);
                let step = if key.code == KeyCode::Left { -1 } else { 1 };
                let next = (current + step).rem_euclid(positions);
                screen.heirloom = (next > 0).then(|| next as usize - 1);
            }
            Transition::Stay
        }
        KeyCode::Enter => raise(screen, ctx),
        _ => Transition::Stay,
    }
}

fn raise(screen: &mut CreationScreen, ctx: &mut AppContext) -> Transition {
    if let Some(problem) = screen.draft.problem(Step::Review) {
        screen.note = Some(format!("Not yet: {problem}."));
        return Transition::Stay;
    }
    let Created { character, gift } = match screen.draft.clone().build() {
        Ok(created) => created,
        Err(e) => {
            screen.note = Some(format!("Not yet: {e}."));
            return Transition::Stay;
        }
    };
    let name = character.name.clone();
    let mut text = vec![format!(
        "{name} takes up the fight. {}, once a {}, haunted by {}.",
        character.background.sin().title,
        character.background.occupation().title,
        character.background.doom().title
    )];
    let Some(campaign) = ctx.campaign.as_mut() else {
        return Transition::Stay;
    };
    if let Some(fallen) = &screen.successor_of {
        text.push(format!(
            "Raised from the settlement to follow {fallen}: -1 Population (the Settlement \
             Manager will track this)."
        ));
    }
    let heirloom = screen.heirloom.filter(|_| campaign.settings.heirloom);
    match campaign.roster.raise(character, heirloom) {
        Ok(Some(item)) => text.push(format!("Took up the {} as an heirloom.", item.name)),
        Ok(None) => {}
        Err(e) => {
            screen.note = Some(format!("Can't: {e}."));
            return Transition::Stay;
        }
    }
    let mut breakdown = None;
    if let Some((g, anguish)) = gift {
        text.push(format!(
            "Gift of the Unmade: {} (-{} Humanity). {}",
            Lore::Gift.entry(g.gift).title,
            g.humanity_loss,
            anguish_text::describe(&anguish)
        ));
        breakdown = anguish.check.and_then(|c| c.breakdown);
    }
    ctx.record("Character", text.join("\n"));
    ctx.info(format!("{name} is ready."));
    // Back to the character tool, which now shows their sheet: a Breakdown from the Gift's
    // Anguish Check gets resolved first.
    match breakdown {
        Some(b) => Transition::Replace(Box::new(BreakdownScreen::new(b))),
        None => Transition::Pop,
    }
}

pub(super) fn draw(screen: &CreationScreen, frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    detail_box(frame, left, "Who They Are", who(&screen.draft));
    let mut lines = what(&screen.draft);
    let options = heirloom_options(ctx);
    if screen.successor_of.is_some() && !options.is_empty() {
        let pick = screen
            .heirloom
            .and_then(|i| options.get(i))
            .map_or("none", |s| s.as_str());
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("Heirloom  ", theme::title()),
            Span::styled(format!("◀ {pick} ▶"), theme::key()),
        ]));
        lines.push(Line::styled(
            "One item the fallen carried may pass to their successor (house rule).",
            theme::dim(),
        ));
    }
    detail_box(frame, right, "What They Carry", lines);
}

fn who(d: &Draft) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let name = if d.name.trim().is_empty() {
        "(unnamed)"
    } else {
        d.name.trim()
    };
    lines.push(Line::styled(name.to_owned(), theme::key()));
    if let Some(a) = d.attributes() {
        let scores: Vec<_> = Attribute::ALL
            .iter()
            .map(|&x| format!("{} {}", x.abbr(), a.score(x)))
            .collect();
        lines.push(Line::styled(scores.join("  "), theme::text()));
    }
    let feats: Vec<_> = d.feats.iter().map(|f| f.name()).collect();
    let drawbacks: Vec<_> = d.taken_drawbacks().iter().map(|x| x.name()).collect();
    let skills: Vec<_> = d.all_skills().iter().map(|s| s.name()).collect();
    lines.push(list_line("Feats", &feats));
    lines.push(list_line("Drawbacks", &drawbacks));
    lines.push(list_line("Skills", &skills));
    if let Some(g) = d.gift {
        lines.push(list_line(
            "Gift",
            &[Lore::Gift.entry(g.gift).title.as_str()],
        ));
    }
    let entry = |lore: Lore, roll: Option<u8>| roll.map(|r| lore.entry(r).title.clone());
    let background = [
        ("Sin", entry(Lore::Sin, d.sin)),
        ("Shard", entry(Lore::Shard, d.shard)),
        ("Doom", entry(Lore::Doom, d.doom)),
        (
            "Former life",
            d.occupation.map(|r| occupation(r).title.clone()),
        ),
    ];
    for (label, value) in background {
        lines.push(list_line(label, &[value.as_deref().unwrap_or("—")]));
    }
    if let Some(r) = &d.relationship {
        let title = &Lore::Relationship.entry(r.roll).title;
        let with = if r.with.trim().is_empty() {
            String::new()
        } else {
            format!(" with {}", r.with.trim())
        };
        lines.push(list_line("Bond", &[format!("{title}{with}").as_str()]));
    }
    lines
}

fn what(d: &Draft) -> Vec<Line<'static>> {
    let profs: Vec<_> = [&d.weapon_proficiency, &d.armor_proficiency]
        .into_iter()
        .flatten()
        .map(|s| s.as_str())
        .collect();
    let mut gear = vec![
        d.starting_weapon
            .clone()
            .unwrap_or_else(|| "(no weapon)".into()),
        "Gambeson".into(),
    ];
    gear.extend(d.cart.iter().map(|(name, qty)| match qty {
        1 => name.clone(),
        _ => format!("{qty} × {name}"),
    }));
    let gear: Vec<&str> = gear.iter().map(String::as_str).collect();
    vec![
        list_line("Proficient", &profs),
        list_line("Gear", &gear),
        list_line("Rations", &[d.rations_left().to_string().as_str()]),
    ]
}

fn list_line(label: &str, items: &[&str]) -> Line<'static> {
    let value = if items.is_empty() {
        "—".to_owned()
    } else {
        items.join(", ")
    };
    Line::from(vec![
        Span::styled(format!("{label:<12}"), theme::dim()),
        Span::styled(value, theme::text()),
    ])
}
