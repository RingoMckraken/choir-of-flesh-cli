use std::sync::LazyLock;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::tables::RollTable;

/// One row of the Surprise/Unknown Events table (p.85–86).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SurpriseRow {
    pub kind: String,
    pub description: String,
    /// Four columns, picked by a D8: 1–2, 3–4, 5–6, 7–8.
    pub flavors: Vec<String>,
}

/// One row of the Random Names table (p.26–27).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NamePair {
    pub male: String,
    pub female: String,
}

/// One entry of the Undertaking tables (p.76–83): a named situation and its write-up.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UndertakingEntry {
    pub title: String,
    pub text: String,
}

pub(super) struct Tables {
    pub complications: RollTable<String>,
    pub actions: RollTable<String>,
    pub themes: RollTable<String>,
    pub surprise: RollTable<SurpriseRow>,
    pub significance: RollTable<String>,
    pub resolution: RollTable<String>,
    pub reactions: RollTable<String>,
    pub reaction_details: RollTable<String>,
    pub genders: RollTable<String>,
    pub ages: RollTable<String>,
    pub activities: RollTable<String>,
    pub emotions: RollTable<String>,
    pub natures: RollTable<String>,
    pub motivations: RollTable<String>,
    pub descriptors: RollTable<String>,
    pub names: RollTable<NamePair>,
    pub hooks: RollTable<UndertakingEntry>,
    pub developments: RollTable<UndertakingEntry>,
    pub tensions: RollTable<UndertakingEntry>,
    pub climaxes: RollTable<UndertakingEntry>,
}

macro_rules! table {
    ($path:literal) => {
        load(
            $path,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/", $path)),
        )
    };
}

/// The bundled tables, parsed on first use. They're compiled into the binary and checked
/// by tests, so a failure here is a build defect, not a runtime condition.
pub(super) static TABLES: LazyLock<Tables> = LazyLock::new(|| Tables {
    complications: table!("oracle/complications.ron"),
    actions: table!("oracle/actions.ron"),
    themes: table!("oracle/themes.ron"),
    surprise: table!("oracle/surprise_events.ron"),
    significance: table!("oracle/significance.ron"),
    resolution: table!("oracle/action_resolution.ron"),
    reactions: table!("npc/reactions.ron"),
    reaction_details: table!("npc/reaction_details.ron"),
    genders: table!("npc/genders.ron"),
    ages: table!("npc/ages.ron"),
    activities: table!("npc/activities.ron"),
    emotions: table!("npc/emotions.ron"),
    natures: table!("npc/natures.ron"),
    motivations: table!("npc/motivations.ron"),
    descriptors: table!("npc/descriptors.ron"),
    names: table!("npc/names.ron"),
    hooks: table!("oracle/undertaking_hooks.ron"),
    developments: table!("oracle/undertaking_developments.ron"),
    tensions: table!("oracle/undertaking_tensions.ron"),
    climaxes: table!("oracle/undertaking_climaxes.ron"),
});

/// The Random Names table (D100, p.26–27), shared with character creation.
pub fn name_table() -> &'static RollTable<NamePair> {
    &TABLES.names
}

fn load<T: DeserializeOwned>(path: &str, source: &str) -> RollTable<T> {
    RollTable::from_ron(source).unwrap_or_else(|e| panic!("bundled table {path} is invalid: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_table_loads() {
        let t = &*TABLES;
        let sizes = [
            (t.actions.entries().count(), 100),
            (t.themes.entries().count(), 100),
            (t.motivations.entries().count(), 100),
            (t.descriptors.entries().count(), 100),
            (t.names.entries().count(), 50),
            (t.surprise.entries().count(), 8),
            (t.complications.entries().count(), 6),
            (t.hooks.entries().count(), 20),
            (t.developments.entries().count(), 20),
            (t.tensions.entries().count(), 20),
            (t.climaxes.entries().count(), 20),
        ];
        for (actual, expected) in sizes {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn spot_checks_against_the_rulebook() {
        let t = &*TABLES;
        assert_eq!(t.actions.lookup(1).unwrap(), "Antagonize");
        assert_eq!(t.actions.lookup(100).unwrap(), "Help");
        assert_eq!(t.themes.lookup(97).unwrap(), "Sanity");
        assert_eq!(t.motivations.lookup(54).unwrap(), "Gain the upper hand");
        assert_eq!(t.motivations.lookup(97).unwrap(), "Self-improvement");
        assert_eq!(t.descriptors.lookup(96).unwrap(), "Pious");
        assert_eq!(t.names.lookup(62).unwrap().female, "Alix");
        assert_eq!(t.resolution.lookup(9).unwrap(), "Near Miss");
        assert_eq!(t.reactions.lookup(7).unwrap(), "Neutral");
        assert_eq!(t.hooks.lookup(1).unwrap().title, "The Silent Village");
        assert_eq!(t.hooks.lookup(20).unwrap().title, "The Face in the Flesh");
        assert_eq!(t.developments.lookup(13).unwrap().title, "The Choice");
        assert_eq!(t.tensions.lookup(5).unwrap().title, "Ambush!");
        assert_eq!(t.climaxes.lookup(20).unwrap().title, "The War is Upon You");
        assert!(
            t.developments
                .lookup(13)
                .unwrap()
                .text
                .ends_with("And probably turn on you immediately after."),
            "an entry that crosses a page break is joined whole"
        );
    }

    #[test]
    fn undertaking_entries_are_tidy() {
        let t = &*TABLES;
        for table in [&t.hooks, &t.developments, &t.tensions, &t.climaxes] {
            for (range, entry) in table.entries() {
                for s in [&entry.title, &entry.text] {
                    assert!(
                        !s.is_empty() && s.trim() == s && !s.contains("  "),
                        "{} {range:?}: {s:?}",
                        table.name()
                    );
                    assert!(
                        !s.contains(['\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}']),
                        "curly quotes are normalized: {s:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn surprise_rows_have_four_flavors() {
        for (_, row) in TABLES.surprise.entries() {
            assert_eq!(row.flavors.len(), 4, "{}", row.kind);
        }
    }

    #[test]
    fn no_text_is_blank_or_padded() {
        let t = &*TABLES;
        let text_tables = [
            &t.complications,
            &t.actions,
            &t.themes,
            &t.significance,
            &t.resolution,
            &t.reactions,
            &t.reaction_details,
            &t.genders,
            &t.ages,
            &t.activities,
            &t.emotions,
            &t.natures,
            &t.motivations,
            &t.descriptors,
        ];
        for table in text_tables {
            for (range, text) in table.entries() {
                assert!(
                    !text.is_empty() && text.trim() == text && !text.contains("  "),
                    "{} {range:?}: {text:?}",
                    table.name()
                );
            }
        }
    }
}
