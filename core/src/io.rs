//! TODO

use std::{fs, path::Path};

use crate::{
    cryptography::fill_random_bytes,
    dto::{
        entry::{EntryBody, EntryHead},
        group::Group,
        master_key::MasterKey,
    },
    error::PWDuckCoreError,
    model::uuid::Uuid,
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

/// Generate a random UUID for the given path.
pub fn generate_uuid(path: &Path) -> Uuid {
    let mut uuid = vec![0_u8; 16];

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

/// Create the directory structure of a new [Vault](Vault) on the given path.
pub fn create_new_vault_dir(path: &Path) -> Result<(), PWDuckCoreError> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join(GROUPS_DIR))?;
    fs::create_dir_all(path.join(ENTRIES_DIR))?;
    fs::create_dir_all(path.join(ENTRIES_DIR).join(HEAD))?;
    fs::create_dir_all(path.join(ENTRIES_DIR).join(BODY))?;
    Ok(())
}

/// Save the [`EntryHead`](EntryHead) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [Vault](Vault)
///     - The UUID as the identifier of the [`EntryHead`](EntryHead)
///     - The [`EntryHead`](EntryHead) to save
pub fn save_entry_head(
    path: &Path,
    uuid: &str,
    entry_head: &EntryHead,
) -> Result<(), PWDuckCoreError> {
    save_entry(
        &path.join(ENTRIES_DIR).join(HEAD),
        uuid,
        ron::to_string(&entry_head)?,
    )
}

/// Load the [`EntryHead`](EntryHead) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`EntryHead`](EntryHead)
pub fn load_entry_head(path: &Path, uuid: &str) -> Result<EntryHead, PWDuckCoreError> {
    let file_name = sha256::digest(uuid);
    let content = fs::read_to_string(path.join(ENTRIES_DIR).join(HEAD).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Load all [`EntryHead`](EntryHead)s of a vault.
///
/// It expects:
///     The [`Path`](Path) as the location of the [`Vault`](Vault)
pub fn load_all_entry_heads(path: &Path) -> Result<Vec<EntryHead>, PWDuckCoreError> {
    let directory = path.join(ENTRIES_DIR).join(HEAD);

    // TODO: Better error handling
    /*fs::read_dir(directory)?.flatten()
    .map(|file| fs::read_to_string(file.path()))
    .flatten()
    .map(|content| ron::from_str(&content).map_err(|err| err.into()))
    .collect();*/

    fs::read_dir(directory)?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|file| fs::read_to_string(file.path()))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|content| ron::from_str(content).map_err(PWDuckCoreError::from))
        .collect()
}

/// Save the [`EntryBody`](EntryBody) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`EntryBody`](EntryBody)
///     - The [`EntryBody`](EntryBody) to save
pub fn save_entry_body(
    path: &Path,
    uuid: &str,
    entry_body: &EntryBody,
) -> Result<(), PWDuckCoreError> {
    save_entry(
        &path.join(ENTRIES_DIR).join(BODY),
        uuid,
        ron::to_string(entry_body)?,
    )
}

/// Load the [`EntryBody`](EntryBody) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`EntryBody`](EntryBody)
pub fn load_entry_body(path: &Path, uuid: &str) -> Result<EntryBody, PWDuckCoreError> {
    let file_name = sha256::digest(uuid);
    let content = fs::read_to_string(path.join(ENTRIES_DIR).join(BODY).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Save the entry to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the entry
///     - The content of the entry
fn save_entry(path: &Path, uuid: &str, content: String) -> Result<(), PWDuckCoreError> {
    let file_name = sha256::digest(uuid);
    fs::write(path.join(file_name), content)?;
    Ok(())
}

/// Save the [`Group`](Group) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`Group`](Group)
///     - The [`Group`](Group) to save
pub fn save_group(path: &Path, uuid: &str, group: &Group) -> Result<(), PWDuckCoreError> {
    let file_name = sha256::digest(uuid);
    fs::write(
        path.join(GROUPS_DIR).join(file_name),
        ron::to_string(&group)?,
    )?;
    Ok(())
}

/// Load the [`Group`](Group) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`Group`](Group)
pub fn load_group(path: &Path, uuid: &str) -> Result<Group, PWDuckCoreError> {
    let file_name = sha256::digest(uuid);
    let content = fs::read_to_string(path.join(GROUPS_DIR).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Load all [`Group`](Group)s of a vault.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
pub fn load_all_groups(path: &Path) -> Result<Vec<Group>, PWDuckCoreError> {
    let directory = path.join(GROUPS_DIR);

    // TODO: Better error handling
    /*fs::read_dir(directory)?.flatten()
    .map(|file| fs::read_to_string(file.path()))
    .flatten()
    .map(|content| ron::from_str(&content).map_err(|err| err.into()))
    .collect()*/

    fs::read_dir(directory)?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|file| fs::read_to_string(file.path()))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|content| ron::from_str(content).map_err(PWDuckCoreError::from))
        .collect()
}

/// Save the [`MasterKey`](MasterKey) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The [`MasterKey`](MasterKey) to save
pub fn save_masterkey(path: &Path, masterkey: MasterKey) -> Result<(), PWDuckCoreError> {
    fs::write(path.join(MASTERKEY_NAME), ron::to_string(&masterkey)?)?;
    drop(masterkey);
    Ok(())
}

/// Load the [`MasterKey`](MasterKey) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the _`Vault`](Vault)
pub fn load_masterkey(path: &Path) -> Result<MasterKey, PWDuckCoreError> {
    let content = fs::read_to_string(path.join(MASTERKEY_NAME))?;
    Ok(ron::from_str(&content)?)
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

        save_entry_head(&path, "uuid", &head).expect("Saving entry head should not fail");

        let result = load_entry_head(&path, "uuid").expect("Loading entry head should not fail");

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

        let mut heads = vec![
            EntryHead::new("one".into(), "one".into()),
            EntryHead::new("two".into(), "two".into()),
            EntryHead::new("three".into(), "three".into()),
        ];

        heads
            .iter()
            .for_each(|head| save_entry_head(&path, head.iv(), head).expect("Should not fail"));

        let mut results =
            load_all_entry_heads(&path).expect("Loading all entry heads should not fail");

        let compare = |a: &EntryHead, b: &EntryHead| a.iv().partial_cmp(b.iv()).unwrap();
        heads.sort_by(compare);
        results.sort_by(compare);

        results.iter().zip(heads).for_each(|(result, head)| {
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

        save_entry_body(&path, "uuid", &body).expect("Saving entry body should not fail");

        let result = load_entry_body(&path, "uuid").expect("Loading entry body should not fail");

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

        save_group(&path, "uuid", &group).expect("Saving group should not fail");

        let result = load_group(&path, "uuid").expect("Loading group should not fail");

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

        let mut groups = vec![
            Group::new("one".into(), "one".into()),
            Group::new("two".into(), "two".into()),
            Group::new("three".into(), "three".into()),
        ];

        groups
            .iter()
            .for_each(|group| save_group(&path, group.iv(), group).expect("Should not fail"));

        let mut results = load_all_groups(&path).expect("Loading all groups should not fail");

        let compare = |a: &Group, b: &Group| a.iv().partial_cmp(b.iv()).unwrap();
        groups.sort_by(compare);
        results.sort_by(compare);

        results.iter().zip(groups).for_each(|(result, group)| {
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
