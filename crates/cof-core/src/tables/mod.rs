//! Random tables: the "roll a die, look up the range" engine used throughout the game.
//!
//! Table contents live in `cof-core/data/*.ron` and are embedded into the binary at compile
//! time. Each file is a [`RollTable`]:
//!
//! ```ron
//! (
//!     name: "Age",
//!     die: D6,
//!     entries: [
//!         (1, 2, "Young"),
//!         (3, 4, "Middle-aged"),
//!         (5, 6, "Elderly"),
//!     ],
//! )
//! ```

mod table;

pub use table::{RollTable, TableError, TableRoll};
