use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::RngExt;
use ratatui::{
    DefaultTerminal,
    style::{Color, Style},
    symbols::Marker,
    text::Span,
    widgets::canvas::{Canvas, Line as CanvasLine},
};

// ---------------------------------------------------------------------------
// ASCII art — 5-row block font using '#' characters.
//
// IMPORTANT: Canvas y=0 is at the BOTTOM, increasing upward.  Row 0 of each
// art array is the topmost row of the word.  Placement formula:
//   canvas_y = word_y_base - row_index * char_h
// where char_h = 100.0 / terminal_rows.
// ---------------------------------------------------------------------------
// const CHOIR: [&str; 5] = [
//
//     " ***  *   *  ***  *** **** ",
//
//     "*     *   * *   *  *  *   *",
//
//     "*     ***** *   *  *  **** ",
//
//     "*     *   * *   *  *  * *  ",
//
//     " ***  *   *  ***  *** *  * ",
//
// ];
//
// const OF: [&str; 5] = [
//
//     " ***  *****",
//
//     "*   * *    ",
//
//     "*   * **** ",
//
//     "*   * *    ",
//
//     " ***  *    ",
//
// ];
//
// const FLESH: [&str; 5] = [
//
//     "***** *     *****  **** *   *",
//
//     "*     *     *     *     *   *",
//
//     "****  *     ****   ***  *****",
//
//     "*     *     *         * *   *",
//
//     "*     ***** *****  **** *   *",
//
// ];
const CHOIR: [&str; 5] = [
    " ###  #   #  ###  ### #### ",
    "#     #   # #   #  #  #   #",
    "#     ##### #   #  #  #### ",
    "#     #   # #   #  #  # #  ",
    " ###  #   #  ###  ### #  # ",
];

const OF: [&str; 5] = [
    " ###  #####",
    "#   # #    ",
    "#   # #### ",
    "#   # #    ",
    " ###  #    ",
];

const FLESH: [&str; 5] = [
    "##### #     #####  #### #   #",
    "#     #     #     #     #   #",
    "####  #     ####   ###  #####",
    "#     #     #         # #   #",
    "#     ##### #####  #### #   #",
];

// Canvas y for the top row of each word (row 0 of the art array).
const Y_CHOIR: f64 = 80.0;
const Y_OF: f64 = 62.0;
const Y_FLESH: f64 = 44.0;

const TICK_MS: u64 = 60;
const REVEAL_PER_TICK: usize = 4; // characters unveiled per tick (~37/sec)
const MAX_DROPS: usize = 60;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

struct CharPos {
    canvas_x: f64,
    canvas_y: f64,
    ch: char,
}

struct Drop {
    x: f64,        // fixed canvas x tied to a letter position
    y: f64,        // current canvas y, decreases each tick
    origin_y: f64, // canvas y of the letter this drop fell from
    speed: f64,    // canvas units per tick
}

enum Phase {
    Drawing { chars_revealed: usize },
    Bleeding { drops: Vec<Drop> },
}

struct TitleScreen {
    all_chars: Vec<CharPos>,
    phase: Phase,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_chars(width: u16, height: u16) -> Vec<CharPos> {
    let char_w = 100.0 / width as f64;
    let char_h = 100.0 / height as f64;

    let words: &[(&[&str], f64)] = &[(&CHOIR, Y_CHOIR), (&OF, Y_OF), (&FLESH, Y_FLESH)];

    let mut chars = Vec::new();
    for &(art, y_base) in words {
        let max_len = art.iter().map(|r| r.len()).max().unwrap_or(0);
        // Centre the word horizontally in the [0, 100] canvas.
        let x_start = 50.0 - (max_len as f64 * char_w) / 2.0;

        for (row, line) in art.iter().enumerate() {
            let canvas_y = y_base - row as f64 * char_h;
            for (col, ch) in line.chars().enumerate() {
                chars.push(CharPos {
                    canvas_x: x_start + col as f64 * char_w,
                    canvas_y,
                    ch,
                });
            }
        }
    }
    chars
}

// Seed a blood rain: drops fall from the top of the canvas at random x positions.
// Staggered starting y values prevent them all arriving at the same time.
fn init_drops() -> Vec<Drop> {
    let mut rng = rand::rng();
    (0..MAX_DROPS)
        .map(|_| {
            let x = rng.random::<f64>() * 100.0;
            let speed = 0.6 + rng.random::<f64>() * 1.2;
            let offset = rng.random::<f64>() * 100.0; // spread out across full height at startup
            Drop {
                x,
                origin_y: 100.0,
                y: 100.0 - offset,
                speed,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// What the player chose on the title screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleChoice {
    Enter,
    Quit,
}

const PROMPT: &str = "press any key to enter  ·  q to quit";
const Y_PROMPT: f64 = 12.0;

/// Plays the title animation until a key is pressed: `q`/Esc quits, anything else enters.
pub fn run(terminal: &mut DefaultTerminal) -> io::Result<TitleChoice> {
    let size = terminal.size()?;
    let char_w = 100.0 / f64::from(size.width.max(1));
    let prompt_x = 50.0 - PROMPT.chars().count() as f64 * char_w / 2.0;
    let all_chars = build_chars(size.width, size.height);

    let mut screen = TitleScreen {
        all_chars,
        phase: Phase::Drawing { chars_revealed: 0 },
    };

    let tick_dur = Duration::from_millis(TICK_MS);
    let mut last_tick = Instant::now();
    let title_style = Style::default().fg(Color::Rgb(220, 220, 200)); // off-white
    let prompt_style = Style::default().fg(Color::Rgb(120, 120, 110));

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let canvas = Canvas::default()
                .x_bounds([0.0, 100.0])
                .y_bounds([0.0, 100.0])
                .marker(Marker::Braille)
                .paint(|ctx| {
                    // Determine how many characters to render.
                    let limit = match &screen.phase {
                        Phase::Drawing { chars_revealed } => *chars_revealed,
                        Phase::Bleeding { .. } => screen.all_chars.len(),
                    };

                    // Render the title (up to `limit` characters).
                    for c in screen.all_chars.iter().take(limit) {
                        if c.ch != ' ' {
                            ctx.print(
                                c.canvas_x,
                                c.canvas_y,
                                Span::styled(c.ch.to_string(), title_style),
                            );
                        }
                    }

                    // Once the title is fully drawn, invite the player in.
                    if matches!(screen.phase, Phase::Bleeding { .. }) {
                        ctx.print(prompt_x, Y_PROMPT, Span::styled(PROMPT, prompt_style));
                    }

                    // Render blood drops (bleeding phase only).
                    if let Phase::Bleeding { drops } = &screen.phase {
                        for drop in drops {
                            // Tapered teardrop: dark tail → bright tip.
                            ctx.draw(&CanvasLine {
                                x1: drop.x,
                                y1: drop.y + 1.5,
                                x2: drop.x,
                                y2: drop.y + 0.5,
                                color: Color::Rgb(120, 0, 0),
                            });
                            ctx.draw(&CanvasLine {
                                x1: drop.x,
                                y1: drop.y + 0.5,
                                x2: drop.x,
                                y2: drop.y,
                                color: Color::Red,
                            });
                        }
                    }
                });

            f.render_widget(canvas, area);
        })?;

        // Input: q or Esc quits, any other key enters.
        let timeout = tick_dur.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            return Ok(match key.code {
                KeyCode::Char('q') | KeyCode::Esc => TitleChoice::Quit,
                _ => TitleChoice::Enter,
            });
        }

        // Tick: advance state.
        if last_tick.elapsed() >= tick_dur {
            let total = screen.all_chars.len();

            // Compute the next state (and signal a phase transition if needed).
            let should_transition = match &mut screen.phase {
                Phase::Drawing { chars_revealed } => {
                    *chars_revealed = (*chars_revealed + REVEAL_PER_TICK).min(total);
                    *chars_revealed >= total
                }
                Phase::Bleeding { drops } => {
                    let mut rng = rand::rng();
                    for drop in drops.iter_mut() {
                        drop.y -= drop.speed;
                        if drop.y < 5.0 {
                            drop.y = drop.origin_y;
                            drop.speed = 0.4 + rng.random::<f64>() * 0.6;
                        }
                    }
                    false
                }
            };

            if should_transition {
                let drops = init_drops();
                screen.phase = Phase::Bleeding { drops };
            }

            last_tick = Instant::now();
        }
    }
}
