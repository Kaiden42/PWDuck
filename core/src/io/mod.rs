//! TODO

use std::path::Path;

use crate::{
    cryptography::fill_random_bytes,
    model::uuid::{self, Uuid},
};

/// The directory name of the groups
pub const GROUPS_DIR: &str = "groups";

/// The directory name of the entries
pub const ENTRIES_DIR: &str = "entries";

/// The directory name of the entry heads.
pub const HEAD: &str = "head";

/// The directory name of the entry bodies.
pub const BODY: &str = "body";

/// The file name of the master key
pub const MASTERKEY_NAME: &str = "masterkey.pwduck";

/// TODO
pub const APPLICATION_SETTINGS_DIR: &str = "PWDuck";
/// TODO
pub const APPLICATION_SETTINGS_NAME: &str = "settings.ron";

mod entry;
pub use entry::*;

mod group;
pub use group::*;

mod master_key;
pub use master_key::*;

mod settings;
pub use settings::*;

mod vault;
pub use vault::*;

/// Generate a random UUID for the given path.
pub fn generate_uuid(path: &Path) -> Uuid {
    let mut uuid = [0_u8; uuid::SIZE];

    loop {
        fill_random_bytes(&mut uuid);
        // TODO: remove code duplication
        let file_name = base64::encode(sha256::digest_bytes(&uuid));

        if !path.join(GROUPS_DIR).join(&file_name).exists()
            && !path.join(ENTRIES_DIR).join(&file_name).exists()
        {
            break;
        }
    }

    uuid.into()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use crate::{
        dto::{
            entry::{EntryBody, EntryHead},
            group::Group,
            master_key::MasterKey,
        },
        io::{load_all_entry_heads, load_all_groups},
        model::uuid,
        Uuid,
    };

    use super::{
        create_new_vault_dir, load_entry_body, load_entry_head, load_group, load_masterkey,
        save_entry_body, save_entry_head, save_group, save_masterkey, BODY, ENTRIES_DIR,
        GROUPS_DIR, HEAD,
    };

    fn remove_test_dir(path: &Path) {
        fs::remove_dir_all(&path).expect("Removing test dir should not fail");
    }

    #[test]
    fn test_create_new_vault_dir() {
        let path: PathBuf = "testing_creating_new_vault".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        create_new_vault_dir(&path).expect("Creating new vault dir should not fail");

        assert!(path.exists());
        assert!(path.join(GROUPS_DIR).exists());
        assert!(path.join(ENTRIES_DIR).exists());
        assert!(path.join(ENTRIES_DIR).join(HEAD).exists());
        assert!(path.join(ENTRIES_DIR).join(BODY).exists());

        remove_test_dir(&path);
    }

    #[test]
    fn test_save_and_load_entry_head() {
        let path: PathBuf = "testing_save_and_load_entry_head".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        fs::create_dir_all(path.join(ENTRIES_DIR).join(HEAD)).expect("Should not fail");

        let head = EntryHead::new("iv".into(), "head".into());

        let uuid: Uuid = [0_u8; uuid::SIZE].into();

        save_entry_head(&path, &uuid, &head).expect("Saving entry head should not fail");

        let result = load_entry_head(&path, &uuid).expect("Loading entry head should not fail");

        assert_eq!(head.iv(), result.iv());
        assert_eq!(head.content(), result.content());

        remove_test_dir(&path);
    }

    #[test]
    fn test_load_all_entry_heads() {
        let path: PathBuf = "testing_load_all_entry_heads".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        fs::create_dir_all(path.join(ENTRIES_DIR).join(HEAD)).expect("Should not fail");

        let mut heads: Vec<(EntryHead, Uuid)> = vec![
            (
                EntryHead::new("one".into(), "one".into()),
                [0_u8; uuid::SIZE].into(),
            ),
            (
                EntryHead::new("two".into(), "two".into()),
                [1_u8; uuid::SIZE].into(),
            ),
            (
                EntryHead::new("three".into(), "three".into()),
                [2_u8; uuid::SIZE].into(),
            ),
        ];

        heads
            .iter()
            .for_each(|(head, uuid)| save_entry_head(&path, uuid, head).expect("Should not fail"));

        let mut results =
            load_all_entry_heads(&path).expect("Loading all entry heads should not fail");

        let compare =
            |a: &(EntryHead, Uuid), b: &(EntryHead, Uuid)| a.0.iv().partial_cmp(b.0.iv()).unwrap();
        heads.sort_by(compare);
        let compare = |a: &EntryHead, b: &EntryHead| a.iv().partial_cmp(b.iv()).unwrap();
        results.sort_by(compare);

        results
            .iter()
            .zip(heads)
            .for_each(|(result, (head, uuid))| {
                assert_eq!(head.iv(), result.iv());
                assert_eq!(head.content(), result.content());
            });

        remove_test_dir(&path);
    }

    #[test]
    fn test_save_and_load_entry_body() {
        let path: PathBuf = "testing_save_and_load_entry_body".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        fs::create_dir_all(path.join(ENTRIES_DIR).join(BODY)).expect("Should not fail");

        let body = EntryBody::new("iv".into(), "body".into());
        let uuid: Uuid = [0_u8; uuid::SIZE].into();

        save_entry_body(&path, &uuid, &body).expect("Saving entry body should not fail");

        let result = load_entry_body(&path, &uuid).expect("Loading entry body should not fail");

        assert_eq!(body.iv(), result.iv());
        assert_eq!(body.content(), result.content());

        remove_test_dir(&path);
    }

    #[test]
    fn test_save_and_load_group() {
        let path: PathBuf = "testing_save_and_load_group".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        fs::create_dir_all(path.join(GROUPS_DIR)).expect("Should not fail");

        let group = Group::new("iv".into(), "content".into());
        let uuid: Uuid = [0_u8; uuid::SIZE].into();

        save_group(&path, &uuid, &group).expect("Saving group should not fail");

        let result = load_group(&path, &uuid).expect("Loading group should not fail");

        assert_eq!(group.iv(), result.iv());
        assert_eq!(group.content(), result.content());

        remove_test_dir(&path)
    }

    #[test]
    fn test_load_all_groups() {
        let path: PathBuf = "testing_load_all_groups".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        fs::create_dir_all(path.join(GROUPS_DIR)).expect("Should not fail");

        let mut groups: Vec<(Group, Uuid)> = vec![
            (
                Group::new("one".into(), "one".into()),
                [0_u8; uuid::SIZE].into(),
            ),
            (
                Group::new("two".into(), "two".into()),
                [1_u8; uuid::SIZE].into(),
            ),
            (
                Group::new("three".into(), "three".into()),
                [2_u8; uuid::SIZE].into(),
            ),
        ];

        groups
            .iter()
            .for_each(|(group, uuid)| save_group(&path, uuid, group).expect("Should not fail"));

        let mut results = load_all_groups(&path).expect("Loading all groups should not fail");

        let compare =
            |a: &(Group, Uuid), b: &(Group, Uuid)| a.0.iv().partial_cmp(b.0.iv()).unwrap();
        groups.sort_by(compare);
        let compare = |a: &Group, b: &Group| a.iv().partial_cmp(b.iv()).unwrap();
        results.sort_by(compare);

        results
            .iter()
            .zip(groups)
            .for_each(|(result, (group, uuid))| {
                assert_eq!(group.iv(), result.iv());
                assert_eq!(group.iv(), result.iv());
            });

        remove_test_dir(&path);
    }

    #[test]
    fn test_save_and_load_masterkey() {
        let path: PathBuf = "testing_save_and_load_masterkey".into();

        if path.exists() {
            remove_test_dir(&path);
        }

        fs::create_dir_all(path.join(GROUPS_DIR)).expect("Should not fail");

        let masterkey = MasterKey::new("salt".into(), "iv".into(), "encrypted_key".into());

        save_masterkey(&path, masterkey.clone()).expect("Saving master key should not fail");

        let result = load_masterkey(&path).expect("Loading master key should not fail");

        assert_eq!(masterkey.salt(), result.salt());
        assert_eq!(masterkey.iv(), result.iv());
        assert_eq!(masterkey.encrypted_key(), result.encrypted_key());

        remove_test_dir(&path);
    }
}
