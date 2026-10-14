use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::{Drawback, Feat, Skill};
use crate::tables::RollTable;

/// A table entry with a name and its write-up: a Sin, a Shard, a Gift…
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TitledEntry {
    pub title: String,
    pub text: String,
}

/// One Former Occupation (p.24–25): pick one of its Skills, and note its Trouble.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OccupationEntry {
    pub title: String,
    pub text: String,
    pub skills: Vec<Skill>,
    pub trouble: String,
}

pub(super) struct Descriptions {
    pub skills: HashMap<Skill, String>,
    pub feats: HashMap<Feat, String>,
    pub drawbacks: HashMap<Drawback, String>,
}

pub(super) struct Tables {
    pub sins: RollTable<TitledEntry>,
    pub shards: RollTable<TitledEntry>,
    pub dooms: RollTable<TitledEntry>,
    pub occupations: RollTable<OccupationEntry>,
    pub relationships: RollTable<TitledEntry>,
    pub gifts: RollTable<TitledEntry>,
    pub stigmata: RollTable<TitledEntry>,
}

macro_rules! data {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/character/",
            $path
        ))
    };
}

/// Bundled data, parsed on first use. It's compiled in and checked by tests, so a parse
/// failure is a build defect.
pub(super) static DESCRIPTIONS: LazyLock<Descriptions> = LazyLock::new(|| Descriptions {
    skills: parse("skills.ron", data!("skills.ron")),
    feats: parse("feats.ron", data!("feats.ron")),
    drawbacks: parse("drawbacks.ron", data!("drawbacks.ron")),
});

pub(super) static TABLES: LazyLock<Tables> = LazyLock::new(|| Tables {
    sins: table("sins.ron", data!("sins.ron")),
    shards: table("shards.ron", data!("shards.ron")),
    dooms: table("dooms.ron", data!("dooms.ron")),
    occupations: table("occupations.ron", data!("occupations.ron")),
    relationships: table("relationships.ron", data!("relationships.ron")),
    gifts: table("gifts.ron", data!("gifts.ron")),
    stigmata: table("stigmata.ron", data!("stigmata.ron")),
});

fn parse<T: DeserializeOwned>(path: &str, source: &str) -> T {
    ron::from_str(source).unwrap_or_else(|e| panic!("bundled data {path} is invalid: {e}"))
}

fn table<T: DeserializeOwned>(path: &str, source: &str) -> RollTable<T> {
    RollTable::from_ron(source).unwrap_or_else(|e| panic!("bundled table {path} is invalid: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_loads_with_the_right_size() {
        let t = &*TABLES;
        let titled = [
            (&t.sins, 10),
            (&t.shards, 10),
            (&t.dooms, 10),
            (&t.relationships, 10),
            (&t.gifts, 20),
            (&t.stigmata, 20),
        ];
        for (table, size) in titled {
            assert_eq!(table.entries().count(), size, "{}", table.name());
            for (range, e) in table.entries() {
                for s in [&e.title, &e.text] {
                    assert!(
                        !s.is_empty() && s.trim() == s && !s.contains("  "),
                        "{} {range:?}: {s:?}",
                        table.name()
                    );
                }
            }
        }
        assert_eq!(t.occupations.entries().count(), 10);
    }

    #[test]
    fn spot_checks_against_the_rulebook() {
        let t = &*TABLES;
        assert_eq!(t.sins.lookup(10).unwrap().title, "Doubt");
        assert_eq!(
            t.shards.lookup(5).unwrap().title,
            "A Page of Plato's Republic"
        );
        assert_eq!(t.dooms.lookup(9).unwrap().title, "The Black Bell");
        assert_eq!(
            t.relationships.lookup(3).unwrap().title,
            "Master & Apprentice"
        );
        assert_eq!(t.gifts.lookup(13).unwrap().title, "Corpulent Storage");
        assert_eq!(t.stigmata.lookup(20).unwrap().title, "Unstable Face");
        assert!(
            t.gifts
                .lookup(9)
                .unwrap()
                .text
                .ends_with("attacks from behind.")
        );
        let soldier = t.occupations.lookup(4).unwrap();
        assert_eq!(
            soldier.skills,
            [
                Skill::BattleFrenzy,
                Skill::PrecisionStrikes,
                Skill::MentalFortitude
            ]
        );
        assert!(
            soldier
                .trouble
                .starts_with("You are trained to follow orders")
        );
    }

    #[test]
    fn occupation_errata_are_applied() {
        let t = &*TABLES;
        // "Disease Resistance" and "Quick Feet" don't exist (Patrick, 2026-10-12).
        assert_eq!(
            t.occupations.lookup(9).unwrap().skills,
            [Skill::Healing, Skill::MindOverMatter]
        );
        assert_eq!(
            t.occupations.lookup(10).unwrap().skills,
            [Skill::Hardy, Skill::LipReading]
        );
        for (_, occupation) in t.occupations.entries() {
            assert!(!occupation.text.contains("Skills:"), "{}", occupation.title);
        }
    }
}
