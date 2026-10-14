//! The pack: equip, stack, add from the catalog or as custom loot, and drop.

use cof_core::character::{Character, Item};
use cof_core::gear::{self, Weight};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};

use super::gear_text;
use crate::context::AppContext;
use crate::screen::{Screen, Transition};
use crate::theme;
use crate::widgets::{Menu, TextInput, detail_box, page, render_menu};

enum Mode {
    Browse,
    Catalog(Menu),
    Custom { name: TextInput, weight: usize },
}

pub struct InventoryScreen {
    menu: Menu,
    mode: Mode,
}

impl Default for InventoryScreen {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            mode: Mode::Browse,
        }
    }
}

fn items(ctx: &AppContext) -> &[Item] {
    ctx.character()
        .map(|c| c.inventory.items.as_slice())
        .unwrap_or_default()
}

impl InventoryScreen {
    fn browse_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        let len = items(ctx).len();
        if self.menu.handle_key(key, len) {
            return Transition::Stay;
        }
        let index = self.menu.selected(len);
        let Some(c) = ctx.character_mut() else {
            return Transition::Pop;
        };
        match (key.code, index) {
            (KeyCode::Esc | KeyCode::Char('q'), _) => return Transition::Pop,
            (KeyCode::Char('a'), _) => self.mode = Mode::Catalog(Menu::default()),
            (KeyCode::Char('c'), _) => {
                self.mode = Mode::Custom {
                    name: TextInput::new(40),
                    weight: 1,
                }
            }
            (KeyCode::Enter | KeyCode::Char('e'), Some(i)) => {
                if !c.inventory.toggle_equipped(i) {
                    ctx.error("Two Backpacks is the most anyone can wear.");
                }
            }
            (KeyCode::Char('+' | '='), Some(i)) => {
                let stack = &mut c.inventory.items[i];
                stack.quantity = stack.quantity.saturating_add(1);
            }
            (KeyCode::Char('-' | '_'), Some(i)) => {
                c.inventory.take(i, 1);
            }
            (KeyCode::Char('d'), Some(i)) => {
                let all = c.inventory.items[i].quantity;
                if let Some(item) = c.inventory.take(i, all) {
                    ctx.record(
                        "Character",
                        format!("Dropped {} × {}.", item.quantity, item.name),
                    );
                }
            }
            (KeyCode::Char('m'), Some(i)) => {
                let item = &mut c.inventory.items[i];
                item.masterwork = !item.masterwork;
            }
            _ => {}
        }
        ctx.save_campaign();
        Transition::Stay
    }

    fn catalog_key(&mut self, key: KeyEvent, ctx: &mut AppContext) {
        let Mode::Catalog(menu) = &mut self.mode else {
            return;
        };
        let catalog = gear::catalog();
        if menu.handle_key(key, catalog.len()) {
            return;
        }
        match key.code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Enter => {
                let Some(g) = menu.selected(catalog.len()).map(|i| &catalog[i]) else {
                    return;
                };
                if let Some(c) = ctx.character_mut() {
                    c.inventory.add(Item::from_catalog(g, 1));
                }
                ctx.save_campaign();
                ctx.info(format!("Added a {}.", g.name));
            }
            _ => {}
        }
    }

    fn custom_key(&mut self, key: KeyEvent, ctx: &mut AppContext) {
        let Mode::Custom { name, weight } = &mut self.mode else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Left => *weight = (*weight + Weight::ALL.len() - 1) % Weight::ALL.len(),
            KeyCode::Right => *weight = (*weight + 1) % Weight::ALL.len(),
            KeyCode::Enter => {
                let text = name.value().trim().to_owned();
                if text.is_empty() {
                    ctx.error("Name the item first.");
                    return;
                }
                let item = Item::custom(text.clone(), Weight::ALL[*weight], 1);
                if let Some(c) = ctx.character_mut() {
                    c.inventory.add(item);
                }
                ctx.record("Character", format!("Found: {text}."));
                self.mode = Mode::Browse;
            }
            _ => {
                name.handle_key(key);
            }
        }
    }
}

impl Screen for InventoryScreen {
    fn handle_key(&mut self, key: KeyEvent, ctx: &mut AppContext) -> Transition {
        match self.mode {
            Mode::Browse => return self.browse_key(key, ctx),
            Mode::Catalog(_) => self.catalog_key(key, ctx),
            Mode::Custom { .. } => self.custom_key(key, ctx),
        }
        Transition::Stay
    }

    fn draw(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let Some(c) = ctx.character() else {
            return;
        };
        let help: &[(&str, &str)] = match self.mode {
            Mode::Browse => &[
                ("e", "equip"),
                ("+/-", "quantity"),
                ("d", "drop"),
                ("m", "masterwork"),
                ("a", "add gear"),
                ("c", "add custom"),
                ("esc", "back"),
            ],
            Mode::Catalog(_) => &[("↑↓", "select"), ("enter", "add one"), ("esc", "done")],
            Mode::Custom { .. } => &[
                ("type", "name"),
                ("←→", "weight"),
                ("enter", "add"),
                ("esc", "cancel"),
            ],
        };
        let body = page(frame, area, &format!("{}'s Pack", c.name), help);
        let [header, _, main] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(body);
        frame.render_widget(header_line(c), header);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                .areas(main);
        match &self.mode {
            Mode::Browse => {
                let rows = c.inventory.items.iter().map(item_line).collect();
                render_menu(frame, left, self.menu, rows);
                let selected = self.menu.selected(c.inventory.items.len());
                let lines = selected.map_or_else(
                    || vec![Line::styled("Nothing carried.", theme::dim())],
                    |i| item_detail(c, &c.inventory.items[i]),
                );
                detail_box(frame, right, "", lines);
            }
            Mode::Catalog(menu) => {
                let rows = gear::catalog()
                    .iter()
                    .map(|g| Line::styled(format!("{:<28}{:>4} ☈", g.name, g.cost), theme::text()))
                    .collect();
                render_menu(frame, left, *menu, rows);
                if let Some(g) = menu
                    .selected(gear::catalog().len())
                    .map(|i| &gear::catalog()[i])
                {
                    detail_box(frame, right, &g.name, gear_text::describe(g));
                }
            }
            Mode::Custom { name, weight } => {
                let mut spans = vec![Span::styled("Name    ", theme::text())];
                spans.extend(name.line().spans);
                let lines = vec![
                    Line::from(spans),
                    Line::from(vec![
                        Span::styled("Weight  ", theme::text()),
                        Span::styled(format!("◀ {} ▶", Weight::ALL[*weight].name()), theme::key()),
                    ]),
                    Line::default(),
                    Line::styled(
                        "Loot, relics, apocalyptic items… Light things bundle 10 to an Item \
                         Slot, Heavy ones take 2 (p.208). Notes and effects are yours to track.",
                        theme::dim(),
                    ),
                ];
                detail_box(frame, left, "Custom Item", lines);
            }
        }
    }
}

fn header_line(c: &Character) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            format!("Item Slots {}/{}", c.inventory.slots_used(), c.item_slots()),
            theme::key(),
        ),
        Span::styled(
            format!(
                "   Rations {} ({} slots)   Backpacks {}",
                c.inventory.rations,
                Weight::Light.slots(c.inventory.rations),
                c.inventory.backpacks()
            ),
            theme::text(),
        ),
    ];
    if c.is_encumbered() {
        spans.push(Span::styled(
            "   ENCUMBERED: Disadvantage on physical rolls",
            theme::error(),
        ));
    }
    Line::from(spans)
}

fn item_line(item: &Item) -> Line<'static> {
    let mut spans = vec![
        Span::styled(if item.equipped { "[E] " } else { "    " }, theme::info()),
        Span::styled(format!("{:<26}", item.name), theme::text()),
        Span::styled(format!("×{:<3}", item.quantity), theme::dim()),
        Span::styled(format!(" {} slot", item.slots()), theme::dim()),
    ];
    if item.masterwork {
        spans.push(Span::styled("  Masterwork", theme::key()));
    }
    Line::from(spans)
}

fn item_detail(c: &Character, item: &Item) -> Vec<Line<'static>> {
    let mut lines = match item.gear() {
        Some(g) => {
            let mut lines = gear_text::describe(g);
            let needs = g.weapon().is_some() || g.armor().is_some_and(|a| a.needs_proficiency());
            if needs && !c.is_proficient(&g.name) {
                lines.push(Line::styled(
                    "Not proficient: attacks (or defensive checks) with Disadvantage.",
                    theme::error(),
                ));
            }
            lines
        }
        None => vec![Line::styled(
            format!("Custom item. {}.", item.weight.name()),
            theme::text(),
        )],
    };
    if item.masterwork {
        lines.push(Line::styled(
            "Masterwork: +1 Mastery to attacks, or to defensive checks for armor (p.219).",
            theme::key(),
        ));
    }
    lines
}
