use super::*;
use crate::rules::TnTable;

fn store() -> (tempfile::TempDir, CampaignStore) {
    let root = tempfile::tempdir().unwrap();
    let store = CampaignStore::at(root.path().join("campaigns"));
    (root, store)
}

#[test]
fn slugs_are_file_safe() {
    assert_eq!(slug("Hope's Watch"), "hope-s-watch");
    assert_eq!(slug("  Last   Rest!! "), "last-rest");
    assert_eq!(slug("Éléonore"), "l-onore");
    assert_eq!(slug("???"), "campaign");
}

#[test]
fn missing_folder_lists_nothing() {
    let (_root, store) = store();
    let list = store.list().unwrap();
    assert!(list.campaigns.is_empty() && list.unreadable.is_empty());
}

#[test]
fn create_then_load_round_trips() {
    let (_root, store) = store();
    let mut created = store.create("Last Rest").unwrap();
    assert_eq!(created.id, "last-rest");
    created.settings.tn_table = TnTable::Scaling;
    store.save(&mut created).unwrap();
    assert_eq!(store.load("last-rest").unwrap(), created);
}

#[test]
fn same_name_gets_a_unique_id() {
    let (_root, store) = store();
    let ids: Vec<_> = (0..3)
        .map(|_| store.create("Grey Ford").unwrap().id)
        .collect();
    assert_eq!(ids, ["grey-ford", "grey-ford-2", "grey-ford-3"]);
}

#[test]
fn empty_names_are_rejected() {
    let (_root, store) = store();
    assert!(matches!(store.create("   "), Err(StorageError::EmptyName)));
}

#[test]
fn list_is_most_recent_first() {
    let (_root, store) = store();
    store.create("First Refuge").unwrap();
    let mut second = store.create("Ember Keep").unwrap();
    let mut first = store.load("first-refuge").unwrap();
    store.save(&mut first).unwrap();
    let names: Vec<_> = store
        .list()
        .unwrap()
        .campaigns
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(names, ["First Refuge", "Ember Keep"]);
    store.save(&mut second).unwrap();
    assert_eq!(store.list().unwrap().campaigns[0].name, "Ember Keep");
}

#[test]
fn saving_keeps_a_backup_of_the_previous_version() {
    let (_root, store) = store();
    let mut campaign = store.create("Stone Creek").unwrap();
    campaign.settings.tn_table = TnTable::Scaling;
    store.save(&mut campaign).unwrap();

    let backup = std::fs::read_to_string(store.dir().join("stone-creek.json.bak")).unwrap();
    let previous: Campaign = serde_json::from_str(&backup).unwrap();
    assert_eq!(previous.settings.tn_table, TnTable::Standard);
}

#[test]
fn backups_and_other_files_are_not_listed() {
    let (_root, store) = store();
    let mut campaign = store.create("Quiet Gate").unwrap();
    store.save(&mut campaign).unwrap();
    std::fs::write(store.dir().join("notes.txt"), "hello").unwrap();
    assert_eq!(store.list().unwrap().campaigns.len(), 1);
}

#[test]
fn corrupt_files_are_reported_without_hiding_others() {
    let (_root, store) = store();
    store.create("Saint's Bridge").unwrap();
    std::fs::write(store.dir().join("broken.json"), "{ nope").unwrap();
    let list = store.list().unwrap();
    assert_eq!(list.campaigns.len(), 1);
    assert!(matches!(
        list.unreadable[..],
        [StorageError::Corrupt { .. }]
    ));
}

#[test]
fn newer_formats_are_refused() {
    let (_root, store) = store();
    let mut campaign = store.create("Hollow End").unwrap();
    campaign.format_version = CURRENT_FORMAT_VERSION + 1;
    store.save(&mut campaign).unwrap();
    assert!(matches!(
        store.load("hollow-end"),
        Err(StorageError::NewerFormat { found, .. }) if found == CURRENT_FORMAT_VERSION + 1
    ));
}

#[test]
fn delete_removes_campaign_and_backup() {
    let (_root, store) = store();
    let mut campaign = store.create("Mercy Hold").unwrap();
    store.save(&mut campaign).unwrap();
    store.delete("mercy-hold").unwrap();
    assert!(std::fs::read_dir(store.dir()).unwrap().next().is_none());
    assert!(matches!(
        store.delete("mercy-hold"),
        Err(StorageError::NotFound(_))
    ));
}

#[test]
fn loading_an_unknown_id_is_not_found() {
    let (_root, store) = store();
    assert!(matches!(
        store.load("nowhere"),
        Err(StorageError::NotFound(_))
    ));
}
