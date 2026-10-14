//! The settlement's line of Unbroken: one active character and a memorial of the fallen.
//! When one falls, a successor is raised from the settlement and may take one Heirloom.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Character, Item};

/// How an Unbroken's story ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fate {
    /// Killed: Injuries, Trauma, an Incursion gone wrong… in the player's words.
    Died { cause: String },
    /// Humanity 0, or the Choir's Price, made them a Penitent (p.47, p.66).
    LostToTheChoir,
    /// Humanity 0 made them one of the Unmade (p.47).
    LostToTheFlesh,
    /// Laid down their arms and rejoined the settlement.
    Retired,
}

impl Fate {
    pub fn describe(&self) -> String {
        match self {
            Fate::Died { cause } if cause.trim().is_empty() => "Died".into(),
            Fate::Died { cause } => format!("Died: {}", cause.trim()),
            Fate::LostToTheChoir => "Lost to the Choir".into(),
            Fate::LostToTheFlesh => "Lost to the Flesh".into(),
            Fate::Retired => "Retired".into(),
        }
    }
}

/// The one item a successor took from the fallen (the Heirloom house rule).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heirloom {
    pub item: String,
    /// The successor's name.
    pub to: String,
}

/// A memorial entry. The whole character is kept, so the sheet can still be looked at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fallen {
    pub character: Character,
    pub fate: Fate,
    pub at: DateTime<Utc>,
    #[serde(default)]
    pub heirloom: Option<Heirloom>,
}

impl fmt::Display for Fallen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = &self.character;
        write!(
            f,
            "{}, level {} ({}). {}.",
            c.name,
            c.level,
            c.background.sin().title,
            self.fate.describe()
        )?;
        if let Some(h) = &self.heirloom {
            write!(f, " Passed the {} to {}.", h.item, h.to)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RosterError {
    #[error("there's already an active Unbroken")]
    AlreadyActive,
    #[error("there's no active Unbroken")]
    NoneActive,
    #[error("there's no such item to pass on")]
    NoSuchItem,
    #[error("an heirloom has already been passed on")]
    HeirloomTaken,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Roster {
    #[serde(default)]
    pub active: Option<Character>,
    /// Oldest first.
    #[serde(default)]
    pub fallen: Vec<Fallen>,
}

impl Roster {
    /// Ends the active Unbroken's story and moves them to the memorial.
    pub fn fall(&mut self, fate: Fate) -> Result<&Fallen, RosterError> {
        let character = self.active.take().ok_or(RosterError::NoneActive)?;
        self.fallen.push(Fallen {
            character,
            fate,
            at: Utc::now(),
            heirloom: None,
        });
        Ok(self.fallen.last().expect("just pushed"))
    }

    /// The items the newest fallen carried, if a successor can still take one of them.
    pub fn heirloom_options(&self) -> &[Item] {
        match self.fallen.last() {
            Some(f) if f.heirloom.is_none() => &f.character.inventory.items,
            _ => &[],
        }
    }

    /// Makes `character` the active Unbroken. With `heirloom`, they take one of that item
    /// from the newest fallen's pack (by index into [`Roster::heirloom_options`]), and the
    /// memorial records it. Returns the item taken.
    pub fn raise(
        &mut self,
        mut character: Character,
        heirloom: Option<usize>,
    ) -> Result<Option<Item>, RosterError> {
        if self.active.is_some() {
            return Err(RosterError::AlreadyActive);
        }
        let taken = match heirloom {
            None => None,
            Some(index) => {
                let fallen = self.fallen.last_mut().ok_or(RosterError::NoSuchItem)?;
                if fallen.heirloom.is_some() {
                    return Err(RosterError::HeirloomTaken);
                }
                let item = fallen
                    .character
                    .inventory
                    .take(index, 1)
                    .ok_or(RosterError::NoSuchItem)?;
                fallen.heirloom = Some(Heirloom {
                    item: item.name.clone(),
                    to: character.name.clone(),
                });
                character.inventory.add(item.clone());
                Some(item)
            }
        };
        self.active = Some(character);
        Ok(taken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::unbroken::fixture::character;
    use crate::gear;

    fn named(name: &str) -> Character {
        let mut c = character();
        c.name = name.into();
        c
    }

    #[test]
    fn a_line_of_unbroken() {
        let mut roster = Roster::default();
        assert_eq!(
            roster.fall(Fate::Retired).unwrap_err(),
            RosterError::NoneActive
        );
        let mut godwin = named("Godwin");
        godwin
            .inventory
            .add(Item::custom("Confessor's Glass", gear::Weight::Light, 1));
        godwin
            .inventory
            .add(Item::from_catalog(gear::find("Torch").unwrap(), 3));
        roster.raise(godwin, None).unwrap();
        assert_eq!(
            roster.raise(named("Spare"), None).unwrap_err(),
            RosterError::AlreadyActive
        );

        roster
            .fall(Fate::Died {
                cause: "Trauma".into(),
            })
            .unwrap();
        assert!(roster.active.is_none());
        assert_eq!(roster.heirloom_options().len(), 2);

        let taken = roster
            .raise(named("Frederick Franz"), Some(0))
            .unwrap()
            .unwrap();
        assert_eq!(taken.name, "Confessor's Glass");
        let frederick = roster.active.as_ref().unwrap();
        assert_eq!(frederick.inventory.count("Confessor's Glass"), 1);
        let memorial = roster.fallen[0].to_string();
        assert!(memorial.starts_with("Godwin, level 1 (Wrath). Died: Trauma."));
        assert!(memorial.ends_with("Passed the Confessor's Glass to Frederick Franz."));
        assert!(roster.heirloom_options().is_empty(), "one heirloom only");
    }

    #[test]
    fn an_heirloom_is_a_single_item_from_a_stack() {
        let mut roster = Roster::default();
        let mut c = named("Hilda");
        c.inventory
            .add(Item::from_catalog(gear::find("Torch").unwrap(), 3));
        roster.raise(c, None).unwrap();
        roster.fall(Fate::Retired).unwrap();
        roster.raise(named("Agnes"), Some(0)).unwrap();
        assert_eq!(roster.active.as_ref().unwrap().inventory.count("Torch"), 1);
        assert_eq!(roster.fallen[0].character.inventory.count("Torch"), 2);
        assert_eq!(
            roster.fallen[0].to_string(),
            "Hilda, level 1 (Wrath). Retired. Passed the Torch to Agnes."
        );
    }

    #[test]
    fn survives_a_save() {
        let mut roster = Roster::default();
        roster.raise(named("Godwin"), None).unwrap();
        roster.fall(Fate::LostToTheFlesh).unwrap();
        let json = serde_json::to_string(&roster).unwrap();
        assert_eq!(serde_json::from_str::<Roster>(&json).unwrap(), roster);
    }
}
