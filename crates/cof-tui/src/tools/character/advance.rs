//! Spending Development Points (p.30), and taking bonus Skills owed for reaching 16.

use cof_core::character::{
    ATTRIBUTE_COST, Attribute, Character, FEAT_COST, Feat, PROFICIENCY_COST, SKILL_COST, Skill,
};
use cof_core::gear;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, detail_box, page, render_menu};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    BonusSkill,
    Feat,
    Skill,
    Proficiency,
    Attribute,
}

const KINDS: [Kind; 5] = [
    Kind::BonusSkill,
    Kind::Feat,
    Kind::Skill,
    Kind::Proficiency,
    Kind::Attribute,
];

/// One thing that can be bought.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pick {
    Feat(Feat),
    Skill(Skill),
    Proficiency(String),
    Attribute(Attribute),
}

#[derive(Default)]
pub struct AdvanceScreen {
    menu: Menu,
    /// The open list of options, if one is open.
    picking: Option<(Kind, Menu)>,
}

fn kind_label(kind: Kind, c: &Character) -> String {
    let once = |bought: bool| if bought { " (bought this level)" } else { "" };
    match kind {
        Kind::BonusSkill => {
            let owed: Vec<_> = c.pending_bonus_skills.iter().map(|a| a.abbr()).collect();
            if owed.is_empty() {
                "Bonus Skill (none owed)".into()
            } else {
                format!("Bonus Skill owed: {} (free)", owed.join(", "))
            }
        }
        Kind::Feat => format!(
            "A new Feat: {FEAT_COST} DP{}",
            once(c.bought_this_level.feat)
        ),
        Kind::Skill => format!(
            "A new Skill: {SKILL_COST} DP{}",
            once(c.bought_this_level.skill)
        ),
        Kind::Proficiency => format!("A Proficiency: {PROFICIENCY_COST} DP"),
        Kind::Attribute => format!("+1 to an Attribute: {ATTRIBUTE_COST} DP"),
    }
}

fn options(kind: Kind, c: &Character) -> Vec<Pick> {
    match kind {
        Kind::BonusSkill => Skill::ALL
            .into_iter()
            .filter(|s| c.pending_bonus_skills.contains(&s.attribute()))
            .filter(|&s| c.check_skill(s).is_ok())
            .map(Pick::Skill)
            .collect(),
        Kind::Feat => Feat::ALL
            .into_iter()
            .filter(|&f| !c.has_feat(f))
            .map(Pick::Feat)
            .collect(),
        Kind::Skill => Skill::ALL
            .into_iter()
            .filter(|&s| c.check_skill(s).is_ok())
            .map(Pick::Skill)
            .collect(),
        Kind::Proficiency => gear::weapons()
            .chain(gear::proficiency_armor())
            .filter(|g| !c.is_proficient(&g.name))
            .map(|g| Pick::Proficiency(g.name.clone()))
            .collect(),
        Kind::Attribute => Attribute::ALL.into_iter().map(Pick::Attribute).collect(),
    }
}

fn option_label(option: &Pick, c: &Character) -> String {
    match option {
        Pick::Feat(f) => f.name().to_owned(),
        Pick::Skill(s) => format!("{:<4}{}", s.attribute().abbr(), s.name()),
        Pick::Proficiency(p) => p.clone(),
        Pick::Attribute(a) => {
            let score = c.attributes.score(*a);
            format!("{} {score} → {}", a.name(), score + 1)
        }
    }
}

fn option_text(option: &Pick) -> String {
    match option {
        Pick::Feat(f) => f.description().to_owned(),
        Pick::Skill(s) => s.description().to_owned(),
        Pick::Proficiency(p) => format!("Fight or defend with the {p} without Disadvantage."),
        Pick::Attribute(a) => format!(
            "Raise {}. Reaching 16 also earns a free bonus Skill from the {} list.",
            a.name(),
            a.abbr()
        ),
    }
}

/// Buys it. Returns the journal line, or why it was refused.
fn buy(kind: Kind, option: &Pick, c: &mut Character) -> Result<String, String> {
    let result = match (kind, option) {
        (Kind::BonusSkill, Pick::Skill(s)) => c.take_bonus_skill(*s),
        (_, Pick::Feat(f)) => c.buy_feat(*f),
        (_, Pick::Skill(s)) => c.buy_skill(*s),
        (_, Pick::Proficiency(p)) => c.buy_proficiency(p),
        (_, Pick::Attribute(a)) => c.raise_attribute(*a),
    };
    result.map_err(|e| e.to_string())?;
    Ok(match option {
        Pick::Feat(f) => format!("{} learns the {} Feat.", c.name, f.name()),
        Pick::Skill(s) => format!("{} learns {}.", c.name, s.name()),
        Pick::Proficiency(p) => format!("{} becomes proficient with the {p}.", c.name),
        Pick::Attribute(a) => format!(
            "{}'s {} rises to {}.",
            c.name,
            a.name(),
            c.attributes.score(*a)
        ),
    })
}

impl Screen for AdvanceScreen {
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        let Some(c) = ctx.character_mut() else {
            return Transition::Pop;
        };
        if let Some((kind, menu)) = &mut self.picking {
            let options = options(*kind, c);
            if menu.handle_key(key, options.len()) {
                return Transition::Stay;
            }
            match key.code {
                KeyCode::Esc => self.picking = None,
                KeyCode::Enter => {
                    let Some(option) = menu.selected(options.len()).map(|i| &options[i]) else {
                        return Transition::Stay;
                    };
                    match buy(*kind, option, c) {
                        Ok(text) => {
                            ctx.record("Character", text.clone());
                            ctx.info(text);
                            self.picking = None;
                        }
                        Err(why) => ctx.error(format!("Can't: {why}.")),
                    }
                }
                _ => {}
            }
            return Transition::Stay;
        }
        if self.menu.handle_key(key, KINDS.len()) {
            return Transition::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Transition::Pop,
            KeyCode::Enter => {
                let kind = KINDS[self.menu.selected(KINDS.len()).unwrap_or(0)];
                if options(kind, c).is_empty() {
                    ctx.error("Nothing to pick there right now.");
                } else {
                    self.picking = Some((kind, Menu::default()));
                }
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(c) = ctx.character() else {
            return;
        };
        let help: &[(&str, &str)] = match self.picking {
            Some(_) => &[("↑↓", "select"), ("enter", "buy"), ("esc", "back")],
            None => &[("↑↓", "select"), ("enter", "browse"), ("esc", "back")],
        };
        let body = page(frame, area, &format!("Advance {}", c.name), help);
        let [header, _, main] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(body);
        frame.render_widget(
            Line::from(vec![
                Span::styled(format!("Level {}   ", c.level), theme::key()),
                Span::styled(format!("Development Points: {}", c.dp), theme::key()),
                Span::styled(
                    "   (6 per level. Feats and Skills: one of each per level.)",
                    theme::dim(),
                ),
            ]),
            header,
        );
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main);
        match &self.picking {
            None => {
                let rows = KINDS
                    .iter()
                    .map(|&k| Line::styled(kind_label(k, c), theme::text()))
                    .collect();
                render_menu(frame, left, self.menu, rows);
                detail_box(
                    frame,
                    right,
                    "",
                    vec![Line::styled(
                        "XP comes in on the sheet (Experience row, End Session). Each level \
                         brings 6 Development Points to spend here (p.30). Saving them up is \
                         fine.",
                        theme::dim(),
                    )],
                );
            }
            Some((kind, menu)) => {
                let options = options(*kind, c);
                let rows = options
                    .iter()
                    .map(|o| Line::styled(option_label(o, c), theme::text()))
                    .collect();
                render_menu(frame, left, *menu, rows);
                if let Some(o) = menu.selected(options.len()).map(|i| &options[i]) {
                    detail_box(
                        frame,
                        right,
                        &kind_label(*kind, c),
                        vec![Line::styled(option_text(o), theme::text())],
                    );
                }
            }
        }
    }
}
