# choir-of-flesh-cli
An Unofficial Terminal UI Collection of Campaign Tools for Black Oath Entertainments (Alex T's) Choir of Flesh TTRPG 

> **Status: early development.** You can found settlements (campaigns) that are saved between
> sessions and play with the **Oracle & Table Roller**. The **Character Creator & Sheet** is in
> progress. The core rules engine (dice, checks, Humanity, Anguish, Trauma,
> Conditions) is in place. The tools below are being built one milestone at a time.

## What Is This?

*Choir of Flesh* is designed to be played solo, which makes you both the Unbroken and the Game
Master. This project is a toolbox of **table aids** that take over the bookkeeping (random
tables, trackers, procedures, enemy behavior) so you can focus on the story. It is not a
full digital port of the game, and you'll still want the rulebook at hand.

## Features

| Tool | What it does | Status |
|---|---|---|
| **Oracle & Table Roller** | Dice roller, Yes/No Oracle, Action & Theme, Surprise Events, Anything of Interest, Random Action Resolution, NPC generator, Undertaking generator, Usage Dice trackers, all recorded in a campaign journal | **Available** |
| **Character Creator & Sheet** | Guided creation (roll or choose every step), a live sheet tracking Humanity, Anguish (with Anguish Checks and Breakdowns), Trauma, Vigor, XP, inventory, and leveling, plus a memorial of the fallen and successors who can carry an heirloom | In progress |
| **Encounter Generator** | The full encounter procedure, from encounter check to creature, difficulty, and party scaling, with complete stat blocks | Planned |
| **Battle Map Simulator** | 5x5 combat grid with terrain, deployment, initiative, attack and injury resolution, and enemy AI driven by the rulebook's behavior tables | Planned |
| **Endless Travail** | A persistent pointcrawl map: point generation, Exploration Turns, foraging, Rumors, and Areas of Corruption with the Taint Track | Planned |
| **Incursion Explorer** | Generates the 3x3 Location with Points of Interest, Dangerous Terrain, and the Extraction Zone, and tracks your Incursion Action budget | Planned |
| **Settlement Manager** | Resources, structures, the six-step Settlement Phase, events, and Notable Citizens | Planned |

All tools share one saved **campaign**, built around your **settlement**. In Choir of
Flesh the settlement is the real protagonist: Unbroken die, and a successor rises from the
population to carry on. Rumors gathered on the Travail unlock Incursions, Incursion loot
flows into your Settlement, and the line of Unbroken goes on for as long as it survives.

## Roadmap

- [x] **Milestone 0 — Workspace:** project split into a rules library (`cof-core`) and the terminal app (`cof-tui`)
- [x] **Milestone 1 — Rules Core:** checks and Target Numbers (standard and scaling tables), Humanity, Anguish and Breakdowns, Trauma, Conditions, Usage Dice
- [x] **Milestone 2 — App Shell:** title screen → main menu → tools, campaign save/load, settings
- [x] **Milestone 3 — Oracle & Table Roller**
- [ ] **Milestone 4 — Character Creator & Sheet** (in progress)
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

The title screen waits for any key (`q` quits). Every screen lists its keys along the bottom
edge. Arrow keys move, `Enter` opens, `Esc` goes back, and `Ctrl+C` quits from anywhere.

### Where Saves Live

By default campaigns are saved in your system's data folder:

| OS | Folder |
|---|---|
| macOS | `~/Library/Application Support/choir-of-flesh/campaigns/` |
| Linux | `~/.local/share/choir-of-flesh/campaigns/` |
| Windows | `%APPDATA%\choir-of-flesh\data\campaigns\` |

In **Settings** (`s` on the main menu) you can switch to a `saves` folder in the directory
you run the toolbox from. Saves are plain JSON, and the previous version of each is kept as
a `.bak` file.

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