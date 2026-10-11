mod dice;
mod rules;
mod title_screen;

fn main() {
    title_screen::run().expect("title screen failed");
}
