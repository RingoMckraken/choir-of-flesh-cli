//! The settlement: the heart of a campaign (p.31–39). Only the name generator exists so
//! far. The rest arrives with the Settlement Manager (Milestone 9).

mod name;

pub use name::{NAME_PREFIXES, NAME_SUFFIXES, roll_name, settlement_name};
