//! Everything on the sheet with its full text, plus Blessings and Phobias to record.

use cof_core::character::{Character, Drawback, Feat, Lore, Skill};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, detail_box, page, render_menu};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    Lore(Lore),
    Occupation,
    Feat(Feat),
    Drawback(Drawback),
    Skill(Skill),
    Gift(usize),
    Stigma(usize),
    Phobia(usize),
}

fn entries(c: &Character) -> Vec<Entry> {
    let mut v = vec![
        Entry::Lore(Lore::Sin),
        Entry::Lore(Lore::Shard),
        Entry::Lore(Lore::Doom),
    ];
    v.push(Entry::Occupation);
    if c.background.relationship.is_some() {
        v.push(Entry::Lore(Lore::Relationship));
    }
    v.extend(c.feats.iter().map(|&f| Entry::Feat(f)));
    v.extend(c.drawbacks.iter().map(|&d| Entry::Drawback(d)));
    v.extend(c.skills.iter().map(|&s| Entry::Skill(s)));
    v.extend((0..c.gifts.len()).map(Entry::Gift));
    v.extend((0..c.stigmata.len()).map(Entry::Stigma));
    v.extend((0..c.phobias.len()).map(Entry::Phobia));
    v
}

/// (kind, name, full text) for an entry.
fn describe(c: &Character, e: Entry) -> (&'static str, String, String) {
    let b = &c.background;
    match e {
        Entry::Lore(Lore::Relationship) => {
            let entry = b.relationship().expect("listed only when set");
            ("Bond", entry.title.clone(), entry.text.clone())
        }
        Entry::Lore(lore) => {
            let entry = match lore {
                Lore::Sin => b.sin(),
                Lore::Shard => b.shard(),
                _ => b.doom(),
            };
            (lore.name(), entry.title.clone(), entry.text.clone())
        }
        Entry::Occupation => {
            let o = b.occupation();
            (
                "Former life",
                o.title.clone(),
                format!("{}\n\nTrouble: {}", o.text, o.trouble),
            )
        }
        Entry::Feat(f) => ("Feat", f.name().into(), f.description().into()),
        Entry::Drawback(d) => ("Drawback", d.name().into(), d.description().into()),
        Entry::Skill(s) => ("Skill", s.name().into(), s.description().into()),
        Entry::Gift(i) => {
            let g = Lore::Gift.entry(c.gifts[i]);
            ("Gift", g.title.clone(), g.text.clone())
        }
        Entry::Stigma(i) => {
            let s = Lore::Stigma.entry(c.stigmata[i]);
            ("Stigma", s.title.clone(), s.text.clone())
        }
        Entry::Phobia(i) => (
            "Phobia",
            c.phobias[i].clone(),
            "All checks with Disadvantage while around it (p.49). Confront and overcome it \
             to Defy Your Doom: x removes it and 1 Anguish (p.48)."
                .into(),
        ),
    }
}

#[derive(Default)]
pub struct DetailsScreen {
    menu: Menu,
}

impl Screen for DetailsScreen {
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        let Some(c) = ctx.character_mut() else {
            return Transition::Pop;
        };
        let list = entries(c);
        if self.menu.handle_key(key, list.len()) {
            return Transition::Stay;
        }
        let selected = self.menu.selected(list.len()).map(|i| list[i]);
        let text = match (key.code, selected) {
            (KeyCode::Esc | KeyCode::Char('q'), _) => return Transition::Pop,
            (KeyCode::Char('g'), _) => {
                let roll = Lore::Gift.roll();
                c.gifts.push(roll);
                format!(
                    "Gift of the Unmade (D20 = {roll}): {}.",
                    Lore::Gift.entry(roll).title
                )
            }
            (KeyCode::Char('s'), _) => {
                let roll = Lore::Stigma.roll();
                c.stigmata.push(roll);
                format!(
                    "Stigma of the Abyss (D20 = {roll}): {}.",
                    Lore::Stigma.entry(roll).title
                )
            }
            (KeyCode::Char('x'), Some(Entry::Phobia(i))) => {
                let fear = c.phobias.remove(i);
                c.anguish.lose(1);
                format!("Defied their Doom: no longer afraid of {fear}. -1 Anguish.")
            }
            (KeyCode::Char('x'), Some(Entry::Gift(i))) => {
                let roll = c.gifts.remove(i);
                format!(
                    "Struck a Gift from the record: {}.",
                    Lore::Gift.entry(roll).title
                )
            }
            (KeyCode::Char('x'), Some(Entry::Stigma(i))) => {
                let roll = c.stigmata.remove(i);
                format!(
                    "Struck a Stigma from the record: {}.",
                    Lore::Stigma.entry(roll).title
                )
            }
            _ => return Transition::Stay,
        };
        ctx.record("Character", text.clone());
        ctx.info(text);
        Transition::Stay
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(c) = ctx.character() else {
            return;
        };
        let help = [
            ("↑↓", "select"),
            ("g", "roll a Gift"),
            ("s", "roll a Stigma"),
            ("x", "remove"),
            ("esc", "back"),
        ];
        let body = page(frame, area, &format!("{} in Detail", c.name), &help);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(body);
        let list = entries(c);
        let rows = list
            .iter()
            .map(|&e| {
                let (kind, name, _) = describe(c, e);
                Line::from(vec![
                    Span::styled(format!("{kind:<24}"), theme::dim()),
                    Span::styled(name, theme::text()),
                ])
            })
            .collect();
        render_menu(frame, left, self.menu, rows);
        let [text, note] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(6)]).areas(right);
        if let Some(e) = self.menu.selected(list.len()).map(|i| list[i]) {
            let (_, name, body) = describe(c, e);
            let lines = body
                .lines()
                .map(|l| Line::styled(l.to_owned(), theme::text()))
                .collect();
            detail_box(frame, text, &name, lines);
        }
        detail_box(
            frame,
            note,
            "",
            vec![Line::styled(
                "g and s record a Blessing of the Flesh rolled in play. Its price (1 Major \
                 Injury, D4 Humanity, Anguish) is yours to apply for now: Prayers and the Act \
                 of Apostasy come in a later milestone.",
                theme::dim(),
            )],
        );
    }
}
