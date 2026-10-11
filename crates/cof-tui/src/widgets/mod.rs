//! Reusable building blocks for screens.

mod menu;
mod page;
mod text_input;

pub use menu::{Menu, render_menu};
pub use page::{centered, page};
pub use text_input::TextInput;
