mod app;
mod context;
mod screen;
mod screens;
mod theme;
mod title_screen;
mod tools;
mod widgets;

use std::io;

use ratatui::DefaultTerminal;

use crate::app::App;
use crate::context::AppContext;
use crate::title_screen::TitleChoice;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load settings before taking over the terminal, so a fatal error prints normally.
    let ctx = AppContext::load()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, ctx);
    ratatui::restore();
    Ok(result?)
}

fn run(terminal: &mut DefaultTerminal, ctx: AppContext) -> io::Result<()> {
    if title_screen::run(terminal)? == TitleChoice::Enter {
        App::new(ctx).run(terminal)?;
    }
    Ok(())
}
