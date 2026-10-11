# choir-of-flesh-cli
An Unofficial Terminal UI Collection of Campaign Tools for Black Oath Entertainments (Alex T's) Choir of Flesh TTRPG 

> **Status: early development.** Right now the app plays the title screen and the core dice and
> check rules are in place. The tools below are being built one milestone at a time.

## What Is This?

*Choir of Flesh* is designed to be played solo, which makes you both the Unbroken and the Game
Master. This project is a toolbox of **table aids** that take over the bookkeeping (random
tables, trackers, procedures, enemy behavior) so you can focus on the story. It is not a
full digital port of the game, and you'll still want the rulebook at hand.

## Features

| Tool | What it does | Status |
|---|---|---|
| **Oracle & Table Roller** | Dice roller with history, Yes/No Oracle, Action/Theme, Surprise Events, NPC generator, Usage Die tracker, Undertaking generator | Planned |
| **Character Creator & Sheet** | Guided creation (roll or choose every step), a live sheet tracking Humanity, Anguish, Trauma, Vigor, XP, inventory, and leveling | Planned |
| **Encounter Generator** | The full encounter procedure, from encounter check to creature, difficulty, and party scaling, with complete stat blocks | Planned |
| **Battle Map Simulator** | 5x5 combat grid with terrain, deployment, initiative, attack and injury resolution, and enemy AI driven by the rulebook's behavior tables | Planned |
| **Endless Travail** | A persistent pointcrawl map: point generation, Exploration Turns, foraging, Rumors, and Areas of Corruption with the Taint Track | Planned |
| **Incursion Explorer** | Generates the 3x3 Location with Points of Interest, Dangerous Terrain, and the Extraction Zone, and tracks your Incursion Action budget | Planned |
| **Settlement Manager** | Resources, structures, the six-step Settlement Phase, events, and Notable Citizens | Planned |

All tools share one saved **campaign**: Rumors gathered on the Travail unlock Incursions,
Incursion loot flows into your Settlement, and combat Trauma stays on your character.

## Roadmap

- [x] **Milestone 0 — Workspace:** project split into a rules library (`cof-core`) and the terminal app (`cof-tui`)
- [ ] **Milestone 1 — Rules Core:** checks and Target Numbers (standard and scaling tables), Humanity, Anguish and Breakdowns, Trauma, Conditions, Usage Dice
- [ ] **Milestone 2 — App Shell:** title screen → main menu → tools, campaign save/load, settings
- [ ] **Milestone 3 — Oracle & Table Roller**
- [ ] **Milestone 4 — Character Creator & Sheet**
- [ ] **Milestone 5 — Encounter Generator**
- [ ] **Milestone 6 — Battle Map Simulator**
- [ ] **Milestone 7 — Endless Travail**
- [ ] **Milestone 8 — Incursion Explorer**
- [ ] **Milestone 9 — Settlement Manager**

## Installation

### Requirements

- **Rust 1.88 or newer** (edition 2024). Install it with [rustup](https://rustup.rs), or update
  an existing install with `rustup update`.
- A terminal with Unicode and color support (most modern terminals on macOS, Linux, and Windows).

### Run From Source

```sh
git clone https://github.com/RingoMckraken/choir-of-flesh-cli.git
cd choir-of-flesh-cli
cargo run --release -p cof-tui
```

### Install the `choir-of-flesh` Command

```sh
cargo install --path crates/cof-tui
choir-of-flesh
```

This puts the binary in `~/.cargo/bin`, which rustup adds to your `PATH`.

### Controls

Press `q` or `Esc` to quit.

### For Developers

```sh
cargo test --workspace                     # run the test suite
cargo clippy --workspace --all-targets     # lint
cargo fmt --all                            # format
```

## Attribution & Intellectual Property

This project is an unofficial, terminal-based adaptation of *Choir of Flesh*, developed with permission from Alex T of Blackoath Entertainment.

All rights to the original game, including its rules, setting, and associated intellectual property, remain with Blackoath Entertainment.

This project is not affiliated with, endorsed by, or sponsored by Blackoath Entertainment.

The source code in this repository is licensed under the GNU General Public License v3.0 (GPL-3.0). This license applies only to the code contained herein and does not grant any rights to the underlying game or its intellectual property.

## Credits

- Choir of Flesh — Created by Alex T, Blackoath Entertainment