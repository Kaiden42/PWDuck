//! The vault in memory.
use std::{collections::HashMap, path::PathBuf};

use zeroize::Zeroize;

use crate::{
    cryptography::{
        decrypt_master_key, derive_key_protection, generate_chacha20_nonce, generate_master_key,
        generate_salt, unprotect_master_key,
    },
    error::PWDuckCoreError,
    io::{create_new_vault_dir, save_master_key},
    mem_protection::MemKey,
    Uuid,
};

use super::{entry::EntryBody, entry::EntryHead, group::Group, master_key::MasterKey};
use getset::{Getters, MutGetters};

/// The in-memory representation of a vault.
#[derive(Clone, Debug, Getters, MutGetters)]
pub struct Vault {
    /// The masterkey used to encrypt the data of this [`Vault`](Vault)
    #[getset(get = "pub")]
    master_key: MasterKey,

    /// The salt to derive the key to decrypt the in-memory encrypted masterkey.
    #[getset(get = "pub")]
    salt: Vec<u8>,

    /// The nonce used to decrypt the in-memory encrypted masterkey.
    #[getset(get = "pub")]
    nonce: Vec<u8>,

    /// The [`PathBuf`](PathBuf) of the location of this [`Vault`](Vault).
    #[getset(get = "pub")]
    path: PathBuf,

    /// The [`PathBuf`](PathBuf) of the location of the optional key file.
    #[getset(get = "pub")]
    key_file: Option<PathBuf>,

    /// The [`Group`](Group)s of this [`Vault`](Vault).
    #[getset(get = "pub", get_mut = "pub")]
    groups: HashMap<Uuid, Group>,
    /// The children of a group,
    children: HashMap<Uuid, Children>,

    /// The [`EntryHead`](EntryHead)s of this vault.
    #[getset(get = "pub")]
    entries: HashMap<Uuid, EntryHead>,

    /// The encrypted data-transfer-objects (dtos) of unsaved [`EntryBody`](EntryBody)s.
    #[getset(get = "pub")]
    unsaved_entry_bodies: HashMap<Uuid, crate::dto::entry::EntryBody>,

    /// A list of containing all the groups that will be deleted from disk when the [`Vault`](Vault) is saved.
    deleted_groups: Vec<Uuid>,
    /// A list of containing all the entries (head, body) that will be deleted from disk when the [`Vault`](Vault) is saved.
    deleted_entries: Vec<(Uuid, Uuid)>,
}

impl Vault {
    /// Generate a new [`Vault`](Vault).
    ///
    /// It expects:
    ///  - The password to encrypt the masterkey of the new [`Vault`](Vault)
    ///  - The location of the optional key file.
    ///  - The memory key to protect the new generated masterkey of the new [`Vault`](Vault)
    ///  - The path as the location of the new [`Vault`](Vault)
    pub fn generate<P1, P2>(
        password: &str,
        key_file: Option<P1>,
        mem_key: &MemKey,
        path: P2,
    ) -> Result<Self, PWDuckCoreError>
    where
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
    {
        let path = path.into();
        let key_file = key_file.map(std::convert::Into::into);
        create_new_vault_dir(&path)?;

        let master_key_dto = generate_master_key(password, key_file.as_ref().map(|p| p.as_ref()))?;

        let salt = generate_salt();
        let nonce = generate_chacha20_nonce()?;

        let master_key = decrypt_master_key(
            &master_key_dto,
            password,
            key_file.as_ref().map(|p| p.as_ref()),
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;

        let mut vault = Self {
            master_key,
            salt,
            nonce,
            path,
            key_file,
            groups: HashMap::new(),
            children: HashMap::new(),
            entries: HashMap::new(),
            unsaved_entry_bodies: HashMap::new(),
            deleted_groups: Vec::new(),
            deleted_entries: Vec::new(),
        };

        let root = Group::create_root_for(vault.path());
        drop(
            vault
                .children
                .insert(root.uuid().clone(), Children::default()),
        );
        drop(vault.groups_mut().insert(root.uuid().clone(), root));

        save_master_key(vault.path(), master_key_dto)?;
        vault.save(mem_key)?;

        Ok(vault)
    }

    /// Save the vault to disk.
    ///
    /// It expects:
    ///  - The [`MemKey`](MemKey) to decrypt the in-memory encrypted masterkey of the [`Vault`](Vault).
    pub fn save(&mut self, mem_key: &MemKey) -> Result<(), PWDuckCoreError> {
        let path = self.path.clone();
        let mut master_key = unprotect_master_key(
            self.master_key.key(),
            &derive_key_protection(mem_key, &self.salt)?,
            &self.nonce,
        )?;

        let unsaved_entry_bodies_result: Result<(), PWDuckCoreError> = self
            .unsaved_entry_bodies
            .iter()
            .try_for_each(|(uuid, entry_body)| crate::io::save_entry_body(&path, uuid, entry_body));
        if unsaved_entry_bodies_result.is_ok() {
            self.unsaved_entry_bodies.clear();
        }

        let group_result: Result<(), PWDuckCoreError> = self
            .groups
            .iter_mut()
            .filter(|(_, group)| group.is_modified())
            .try_for_each(|(_, group)| group.save(&path, &master_key));

        let entry_result: Result<(), PWDuckCoreError> = self
            .entries
            .iter_mut()
            .filter(|(_, entry)| entry.is_modified())
            .try_for_each(|(_, entry)| entry.save(&path, &master_key));

        master_key.zeroize();

        let delete_group_result: Result<(), PWDuckCoreError> = self
            .deleted_groups
            .iter()
            .try_for_each(|group| crate::io::delete_group(&path, group));
        if delete_group_result.is_ok() {
            self.deleted_groups.clear();
        }

        let delete_entry_result: Result<(), PWDuckCoreError> = self
            .deleted_entries
            .iter()
            .try_for_each(|entry| crate::io::delete_entry(&path, &entry.0, &entry.1));
        if delete_entry_result.is_ok() {
            self.deleted_entries.clear();
        }

        unsaved_entry_bodies_result
            .and(group_result.and(entry_result.and(delete_group_result.and(delete_entry_result))))
    }

    /// Load a [`Vault`](Vault) from disk.
    ///
    /// It expects:
    ///  - The password to decrypt the masterkey of the [`Vault`](Vault)
    ///  - The [`MemKey`] to re-encrypt the decrypted masterkey in memory
    ///  - The path as the location of the vault
    pub fn load<P1, P2>(
        password: &str,
        key_file: Option<P1>,
        mem_key: &MemKey,
        path: P2,
    ) -> Result<Self, PWDuckCoreError>
    where
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
    {
        let path = path.into();
        let key_file = key_file.map(std::convert::Into::into);
        let salt = generate_salt();
        let nonce = generate_chacha20_nonce()?;

        let master_key = MasterKey::load(
            &path,
            password,
            key_file.as_ref().map(|p| p.as_ref()),
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;

        let unprotected_master_key = unprotect_master_key(
            master_key.key(),
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;
        let groups = Group::load_all(&path, &unprotected_master_key)?;
        let entries = EntryHead::load_all(&path, &unprotected_master_key)?;
        drop(unprotected_master_key);

        let mut children: HashMap<Uuid, Children> = HashMap::new();

        for (uuid, group) in &groups {
            if !children.contains_key(uuid) {
                drop(children.insert(uuid.clone(), Children::default()));
            }
            if let Some(parent) = group.parent() {
                children
                    .entry(parent.clone())
                    .or_insert_with(Children::default)
                    .groups_mut()
                    .push(uuid.clone());
            }
        }

        for (uuid, entry) in &entries {
            children
                .entry(entry.parent().clone())
                .or_insert_with(Children::default)
                .entries_mut()
                .push(uuid.clone());
        }

        let vault = Self {
            master_key,
            salt,
            nonce,
            path,
            key_file,
            groups,
            children,
            entries,
            unsaved_entry_bodies: HashMap::new(),
            deleted_groups: Vec::new(),
            deleted_entries: Vec::new(),
        };

        Ok(vault)
    }

    /// Get the name of this [`Vault`](Vault).
    #[must_use]
    pub fn get_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("Name of Vault")
    }

    /// Get the UUID of the root [`Group`](Group) of this [`Vault`](Vault).
    #[must_use]
    pub fn get_root_uuid(&self) -> Option<Uuid> {
        self.groups
            .iter()
            .find(|(_uuid, group)| group.is_root())
            .map(|(_uuid, group)| group.uuid().clone())
    }

    /// Insert a new [`Group`](Group) into this [`Vault`](Vault).
    pub fn insert_group(&mut self, group: Group) {
        // Insert into parent's children.
        let _ = group
            .parent()
            .as_ref()
            .and_then(|parent| self.children.get_mut(parent))
            .filter(|parent| !parent.groups().contains(group.uuid())) // TODO: find better way
            .map(|parent| parent.groups_mut().push(group.uuid().clone()));
        // Add own children.
        drop(
            self.children
                .insert(group.uuid().clone(), Children::default()),
        );
        drop(self.groups.insert(group.uuid().clone(), group));
    }

    /// Delete a [`Group`](Group) from this [`Vault`](Vault).
    pub fn delete_group(&mut self, uuid: &Uuid) {
        if let Some(group) = self.groups.remove(uuid) {
            // Remove from parent's children.
            let _ = group
                .parent()
                .as_ref()
                .and_then(|parent| self.children.get_mut(parent))
                .map(|parent| parent.groups_mut().retain(|e| e != uuid));
            // Remove own children.
            drop(self.children.remove(uuid));
            self.deleted_groups.push(uuid.clone());
        }
    }

    /// Insert a new entry into this [`Vault`](Vault).
    ///
    /// It expects:
    ///  - The [`EntryHead`] of the new entry
    ///  - The [`EntryBody`] of the new entry
    ///  - The masterkey to decrypt the [`EntryBody`](EntryBody)
    pub fn insert_entry(
        &mut self,
        entry_head: EntryHead,
        entry_body: EntryBody,
        master_key: &[u8],
    ) -> Result<(), PWDuckCoreError> {
        // Insert into parent's children.
        let _ = self
            .children
            .get_mut(entry_head.parent())
            .filter(|parent| !parent.entries().contains(entry_head.uuid())) // TODO: find better way
            .map(|parent| parent.entries_mut().push(entry_head.uuid().clone()));
        drop(self.entries.insert(entry_head.uuid().clone(), entry_head));

        drop(
            self.unsaved_entry_bodies
                .insert(entry_body.uuid().clone(), entry_body.encrypt(master_key)?),
        );
        drop(entry_body);
        Ok(())
    }

    /// Delete an entry from this [`Vault`](Vault).
    pub fn delete_entry(&mut self, uuid: &Uuid) {
        if let Some(entry_head) = self.entries.remove(uuid) {
            // Remove from parent's children.
            let _ = self
                .children
                .get_mut(entry_head.parent())
                .map(|parent| parent.entries_mut().retain(|e| e != uuid));
            let entry_body = entry_head.body();

            drop(self.unsaved_entry_bodies.remove(entry_body));

            self.deleted_entries
                .push((uuid.clone(), entry_body.clone()));
        }
    }

    /// Get all [`Group`](Group)s in this [`Vault`] that are the children of the specified parent [`Group`](Group).
    #[must_use]
    pub fn get_groups_of(&self, parent_uuid: &Uuid) -> Vec<&Group> {
        self.children
            .get(parent_uuid)
            .map_or_else(Vec::new, |parent| {
                parent
                    .groups()
                    .iter()
                    .map(|group| &self.groups[group])
                    .collect()
            })
    }

    /// Get all [`EntryHead`](EntryHead)s in this [`Vault`] that are children of the specified parent [`Group`](Group).
    #[must_use]
    pub fn get_entries_of(&self, parent_uuid: &Uuid) -> Vec<&EntryHead> {
        self.children
            .get(parent_uuid)
            .map_or_else(Vec::new, |parent| {
                parent
                    .entries()
                    .iter()
                    .map(|entry| &self.entries[entry])
                    .collect()
            })
    }

    /// Trie, if this [`Vault`](Vault) contains unsaved changes.
    #[must_use]
    pub fn contains_unsaved_changes(&self) -> bool {
        self.groups.iter().any(|(_uuid, group)| group.is_modified())
            || self
                .entries
                .iter()
                .any(|(_uuid, entry)| entry.is_modified())
            || !self.unsaved_entry_bodies.is_empty()
            || !self.deleted_entries.is_empty()
            || !self.deleted_groups.is_empty()
    }

    /// Returns the [`ItemList`](ItemList) containing [`Group`](Group)s and [`EntryHead`](EntryHead) based on the given filters.
    ///
    /// It expects:
    ///  - The UUID of the current selected [`Group`](Group)
    ///  - The optional search filter
    #[must_use]
    pub fn get_item_list_for<'a>(
        &'a self,
        selected_group_uuid: &Uuid,
        search: Option<&str>,
    ) -> ItemList<'a> {
        let (mut groups, mut entries) = search.map_or_else(
            || {
                (
                    self.get_groups_of(selected_group_uuid),
                    self.get_entries_of(selected_group_uuid),
                )
            },
            |search| {
                let search = search.to_lowercase();
                (
                    self.groups
                        .iter()
                        .filter(|(_uuid, group)| group.title().to_lowercase().contains(&search))
                        .map(|(_, group)| group)
                        .collect(),
                    self.entries
                        .iter()
                        .filter(|(_uuid, entry)| entry.title().to_lowercase().contains(&search))
                        .map(|(_, entry)| entry)
                        .collect(),
                )
            },
        );

        groups.sort_by(|&a, &b| a.title().cmp(b.title()));
        entries.sort_by(|&a, &b| a.title().cmp(b.title()));

        ItemList { groups, entries }
    }
}

/// The children of a group.
#[derive(Clone, Debug, Default, Getters, MutGetters)]
pub struct Children {
    /// The list of sub groups.
    #[getset(get = "pub", get_mut = "pub")]
    groups: Vec<Uuid>,
    /// The list of entries.
    #[getset(get = "pub", get_mut = "pub")]
    entries: Vec<Uuid>,
}

/// Filtered collection of [`Group`](Group)s and [`EntryHead`](EntryHead)s.
#[derive(Debug, Getters)]
pub struct ItemList<'a> {
    /// Collection of [`Group`](Group)s.
    #[getset(get = "pub")]
    groups: Vec<&'a Group>,

    /// Collection of [`EntryHead`](EntryHead)s.
    #[getset(get = "pub")]
    entries: Vec<&'a EntryHead>,
}

impl<'a> ItemList<'a> {
    /// Trie, if this [`ItemList`](ItemList) is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use mocktopus::mocking::*;
    use seckey::SecBytes;
    use tempfile::tempdir;

    use crate::{cryptography, model::uuid, EntryBody, EntryHead, Group, MemKey, Uuid};

    use super::{ItemList, Vault};

    const PASSWORD: &str = "This is a totally secure password";
    const VAULT_NAME: &str = "Default Vault";

    fn default_mem_key() -> MemKey {
        MemKey::with_length.mock_safe(|len| {
            MockResult::Return(SecBytes::with(len, |buf| buf.fill(255_u8)).into())
        });
        MemKey::new()
    }

    fn default_vault(path: &Path, mem_key: &MemKey) -> Vault {
        let path = path.join(VAULT_NAME);
        Vault::generate(PASSWORD, Option::<String>::None, &mem_key, path).unwrap()
    }

    #[test]
    fn generate_new_vault() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        cryptography::fill_random_bytes.mock_safe(|buf| {
            buf.fill(42_u8);
            MockResult::Return(())
        });
        let mem_key = default_mem_key();

        let vault = Vault::generate(PASSWORD, Option::<String>::None, &mem_key, &path)
            .expect("Creating new vault should not fail.");

        assert!(path.exists());
        assert!(path.join(crate::io::MASTERKEY_NAME).exists());
        assert_eq!(vault.groups.len(), 1);
        for (uuid, group) in &vault.groups {
            assert!(group.is_root());

            let children = vault
                .children
                .get(uuid)
                .expect("There should be an empty children value for root group.");
            assert!(children.entries.is_empty());
            assert!(children.groups.is_empty());
        }
        assert!(vault.entries.is_empty());
        assert!(vault.unsaved_entry_bodies.is_empty());
        assert!(vault.deleted_groups.is_empty());
        assert!(vault.deleted_entries.is_empty());
    }

    #[test]
    fn save_and_load_vault() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .master_key
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();

        let mut groups: Vec<Group> = (0..=20)
            .into_iter()
            .map(|next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let group = Group::new(uuid, root.clone(), format!("Group: {}", next));
                vault.insert_group(group.clone());
                group
            })
            .collect();

        let mut entries: Vec<(EntryHead, EntryBody)> = (0..=20)
            .into_iter()
            .map(|next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let head = EntryHead::new(
                    uuid.clone(),
                    root.clone(),
                    format!("Entry: {}", next),
                    uuid.clone(),
                );
                let body = EntryBody::new(
                    uuid,
                    format!("Username: {}", next),
                    format!("Password: {}", next),
                );
                vault
                    .insert_entry(head.clone(), body.clone(), &master_key)
                    .unwrap();
                (head, body)
            })
            .collect();

        vault.save(&mem_key).expect("Should not fail");

        groups.sort_by(|a, b| a.title().cmp(b.title()));
        entries.sort_by(|(a, _), (b, _)| a.title().cmp(b.title()));

        // Check if the loaded vault contains all saved items.
        let loaded_vault = Vault::load(
            PASSWORD,
            Option::<String>::None,
            &mem_key,
            &path.join(VAULT_NAME),
        )
        .expect("Should not fail");

        let item_list_for_root = loaded_vault.get_item_list_for(&root, None);
        assert_eq!(item_list_for_root.groups.len(), groups.len());
        assert_eq!(item_list_for_root.entries.len(), entries.len());
        groups
            .iter()
            .zip(item_list_for_root.groups.iter())
            .for_each(|(expected, group)| {
                assert_eq!(expected.title(), group.title());
            });
        entries
            .iter()
            .zip(item_list_for_root.entries.iter())
            .for_each(|((expected_head, expected_body), entry)| {
                assert_eq!(expected_head.title(), entry.title());
                let body =
                    EntryBody::load(&path.join(VAULT_NAME), entry.body(), &master_key).unwrap();
                assert_eq!(expected_body.username(), body.username());
                assert_eq!(expected_body.password(), body.password());
            });

        // Check if the deletion of items works.
        let delete_group_count = 3;
        let delete_entry_count = 7;
        groups.iter().take(delete_group_count).for_each(|group| {
            vault.delete_group(group.uuid());
        });
        entries
            .iter()
            .take(delete_entry_count)
            .for_each(|(entry, _)| {
                vault.delete_entry(entry.uuid());
            });
        vault.save(&mem_key).expect("Should not fail");

        let loaded_vault = Vault::load(
            PASSWORD,
            Option::<String>::None,
            &mem_key,
            &path.join(VAULT_NAME),
        )
        .expect("Should not fail");

        let item_list_for_root = loaded_vault.get_item_list_for(&root, None);
        assert_eq!(
            item_list_for_root.groups.len(),
            groups.len() - delete_group_count
        );
        assert_eq!(
            item_list_for_root.entries.len(),
            entries.len() - delete_entry_count
        );
        groups
            .iter()
            .skip(delete_group_count)
            .zip(item_list_for_root.groups.iter())
            .for_each(|(expected, group)| {
                assert_eq!(expected.title(), group.title());
            });
        entries
            .iter()
            .skip(delete_entry_count)
            .zip(item_list_for_root.entries.iter())
            .for_each(|((expected_head, expected_body), entry)| {
                assert_eq!(expected_head.title(), entry.title());
                let body =
                    EntryBody::load(&path.join(VAULT_NAME), entry.body(), &master_key).unwrap();
                assert_eq!(expected_body.username(), body.username());
                assert_eq!(expected_body.password(), body.password());
            });
    }

    #[test]
    fn get_name() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let vault = default_vault(&path, &mem_key);

        assert_eq!(vault.get_name(), VAULT_NAME);
    }

    #[test]
    fn get_root_uuid() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        cryptography::fill_random_bytes.mock_safe(|buf| {
            buf.fill(42_u8);
            MockResult::Return(())
        });

        let vault = default_vault(&path, &mem_key);

        assert_eq!(
            vault
                .get_root_uuid()
                .expect("There should always be a root uuid"),
            Uuid::from([42_u8; uuid::SIZE]),
        );
    }

    #[test]
    fn insert_group() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let uuid: Uuid = [21_u8; uuid::SIZE].into();
        let title = "Added group";

        let group = Group::new(uuid.clone(), root.clone(), title.to_string());

        vault.insert_group(group);

        assert!(vault.groups.contains_key(&uuid));
        let tmp = vault
            .groups
            .get(&uuid)
            .expect("The newly added group should exist.");

        assert_eq!(tmp.uuid(), &uuid);
        assert_eq!(
            tmp.parent().as_ref().expect("Group should have a parent"),
            &root
        );
        assert_eq!(tmp.title().as_str(), title);

        let root_children = vault.children.get(&root).unwrap();
        assert_eq!(root_children.groups.len(), 1);
        assert!(root_children.groups.contains(&uuid));

        let group_children = vault
            .children
            .get(&uuid)
            .expect("There should be an empty children entry for the group.");
        assert!(group_children.groups.is_empty());
        assert!(group_children.entries.is_empty());
    }

    #[test]
    fn delete_group() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let uuid: Uuid = [21_u8; uuid::SIZE].into();

        let group = Group::new(uuid.clone(), root.clone(), "Title".into());
        vault.insert_group(group);

        assert!(vault.groups().contains_key(&uuid));

        vault.delete_group(&uuid);

        assert!(!vault.groups.contains_key(&uuid));
        assert!(vault.deleted_groups.contains(&uuid));

        let root_children = vault.children.get(&root).unwrap();
        assert!(!root_children.groups.contains(&uuid));

        assert!(vault.children.get(&uuid).is_none());
    }

    #[test]
    fn insert_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .master_key
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();

        let head_uuid: Uuid = [42_u8; uuid::SIZE].into();
        let body_uuid: Uuid = [21_u8; uuid::SIZE].into();
        let title = "Added entry";
        let username = "Username";
        let password = "Password";

        let head = EntryHead::new(
            head_uuid.clone(),
            root.clone(),
            title.to_string(),
            body_uuid.clone(),
        );
        let body = EntryBody::new(
            body_uuid.clone(),
            username.to_string(),
            password.to_string(),
        );

        vault
            .insert_entry(head, body, &master_key)
            .expect("Inserting new entry should not fail.");

        assert!(vault.entries.contains_key(&head_uuid));

        let tmp = vault
            .entries
            .get(&head_uuid)
            .expect("The newly added entry should exist.");
        assert_eq!(tmp.uuid(), &head_uuid);
        assert_eq!(tmp.parent(), &root);
        assert_eq!(tmp.title().as_str(), title);

        let root_children = vault.children.get(&root).unwrap();

        assert!(root_children.entries.contains(&head_uuid));

        assert!(vault.unsaved_entry_bodies.contains_key(&body_uuid));
    }

    #[test]
    fn delete_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .master_key
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();

        let head_uuid: Uuid = [42_u8; uuid::SIZE].into();
        let body_uuid: Uuid = [21_u8; uuid::SIZE].into();

        let head = EntryHead::new(
            head_uuid.clone(),
            root.clone(),
            "Title".to_string(),
            body_uuid.clone(),
        );
        let body = EntryBody::new(
            body_uuid.clone(),
            "Username".to_string(),
            "Password".to_string(),
        );

        vault.insert_entry(head, body, &master_key).unwrap();

        assert!(vault.entries.contains_key(&head_uuid));
        assert!(vault
            .children
            .get(&root)
            .unwrap()
            .entries
            .contains(&head_uuid));
        assert!(vault.unsaved_entry_bodies.contains_key(&body_uuid));
        assert!(!vault
            .deleted_entries
            .contains(&(head_uuid.clone(), body_uuid.clone())));

        vault.delete_entry(&head_uuid);

        assert!(!vault.entries.contains_key(&head_uuid));
        assert!(!vault
            .children
            .get(&root)
            .unwrap()
            .entries
            .contains(&head_uuid));
        assert!(!vault.unsaved_entry_bodies.contains_key(&body_uuid));
        assert!(vault.deleted_entries.contains(&(head_uuid, body_uuid)));
    }

    #[test]
    fn get_groups_of() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let mut expected: Vec<Group> = (0..=10)
            .into_iter()
            .map(|next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let group = Group::new(uuid, root.clone(), format!("Group: {}", next));
                vault.insert_group(group.clone());
                group
            })
            .collect();

        let mut groups = vault.get_groups_of(&root);

        expected.sort_by(|a, b| a.title().cmp(b.title()));
        groups.sort_by(|a, b| a.title().cmp(b.title()));

        assert_eq!(expected.len(), groups.len());
        expected
            .iter()
            .zip(groups.iter())
            .for_each(|(expected, group)| {
                assert_eq!(expected.title(), group.title());
            });
    }

    #[test]
    fn get_entries_of() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .master_key
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();

        let body_uuid: Uuid = [255_u8; uuid::SIZE].into();
        let body = EntryBody::new(body_uuid.clone(), "username".into(), "password".into());

        let mut expected: Vec<EntryHead> = (0..=10)
            .into_iter()
            .map(|next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let head = EntryHead::new(
                    uuid,
                    root.clone(),
                    format!("Entry: {}", next),
                    body_uuid.clone(),
                );
                vault
                    .insert_entry(head.clone(), body.clone(), &master_key)
                    .unwrap();
                head
            })
            .collect();

        let mut entries = vault.get_entries_of(&root);

        expected.sort_by(|a, b| a.title().cmp(b.title()));
        entries.sort_by(|a, b| a.title().cmp(b.title()));

        assert_eq!(expected.len(), entries.len());
        expected
            .iter()
            .zip(entries.iter())
            .for_each(|(expected, entry)| {
                assert_eq!(expected.title(), entry.title());
            });
    }

    #[test]
    fn contains_unsaved_changes() {
        let mem_key = default_mem_key();

        let clear_vault = || {
            let dir = tempdir().unwrap();
            let path = dir.path();

            let vault = default_vault(&path, &mem_key);

            assert!(!vault.contains_unsaved_changes());
            (dir, vault)
        };

        // Contains unsaved group.
        let (_dir, mut vault) = clear_vault();
        let root = vault.get_root_uuid().unwrap();
        vault.insert_group(Group::new(
            [42_u8; uuid::SIZE].into(),
            root.clone(),
            "Group".into(),
        ));
        assert!(vault.contains_unsaved_changes());

        // Contains unsaved entry.
        let (_dir, mut vault) = clear_vault();
        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .master_key
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();
        vault
            .insert_entry(
                EntryHead::new(
                    [42_u8; uuid::SIZE].into(),
                    root.clone(),
                    "Title".into(),
                    [21_u8; uuid::SIZE].into(),
                ),
                EntryBody::new(
                    [21_u8; uuid::SIZE].into(),
                    "username".into(),
                    "password".into(),
                ),
                &master_key,
            )
            .unwrap();
        assert!(vault.contains_unsaved_changes());

        // Contains unsaved group deletion.
        let (_dir, mut vault) = clear_vault();
        let root = vault.get_root_uuid().unwrap();
        let uuid: Uuid = [42_u8; uuid::SIZE].into();
        vault.insert_group(Group::new(uuid.clone(), root.clone(), "Group".into()));
        vault.save(&mem_key).unwrap();
        assert!(!vault.contains_unsaved_changes());
        vault.delete_group(&uuid);
        assert!(vault.contains_unsaved_changes());

        // Contains unsaved entry deletion.
        let (_dir, mut vault) = clear_vault();
        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .master_key
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();
        let head_uuid: Uuid = [42_u8; uuid::SIZE].into();
        let body_uuid: Uuid = [21_u8; uuid::SIZE].into();
        vault
            .insert_entry(
                EntryHead::new(
                    head_uuid.clone(),
                    root.clone(),
                    "Title".into(),
                    body_uuid.clone(),
                ),
                EntryBody::new(body_uuid, "username".into(), "password".into()),
                &master_key,
            )
            .unwrap();
        vault.save(&mem_key).unwrap();
        assert!(!vault.contains_unsaved_changes());
        vault.delete_entry(&head_uuid);
        assert!(vault.contains_unsaved_changes());
    }

    #[test]
    fn get_item_list_for() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let mem_key = default_mem_key();

        let mut vault = default_vault(&path, &mem_key);

        let root = vault.get_root_uuid().unwrap();
        let master_key = vault
            .masterkey
            .as_unprotected(&mem_key, &vault.salt, &vault.nonce)
            .unwrap();

        let mut groups: Vec<Group> = (0..=20)
            .into_iter()
            .map(|next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let group = Group::new(uuid, root.clone(), format!("Group: {}", next));
                vault.insert_group(group.clone());
                group
            })
            .collect();

        let mut entries: Vec<EntryHead> = (0..=20)
            .into_iter()
            .map(|next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let head = EntryHead::new(
                    uuid,
                    root.clone(),
                    format!("Entry: {}", next),
                    [255_u8; uuid::SIZE].into(),
                );
                let body = EntryBody::new(
                    [255_u8; uuid::SIZE].into(),
                    "username".into(),
                    "password".into(),
                );
                vault.insert_entry(head.clone(), body, &master_key).unwrap();
                head
            })
            .collect();

        groups.sort_by(|a, b| a.title().cmp(b.title()));
        entries.sort_by(|a, b| a.title().cmp(b.title()));

        let item_list_for_root = vault.get_item_list_for(&root, None);
        assert_eq!(item_list_for_root.groups.len(), groups.len());
        assert_eq!(item_list_for_root.entries.len(), entries.len());
        groups
            .iter()
            .zip(item_list_for_root.groups.iter())
            .for_each(|(expected, group)| {
                assert_eq!(expected.title(), group.title());
            });
        entries
            .iter()
            .zip(item_list_for_root.entries.iter())
            .for_each(|(expected, entry)| {
                assert_eq!(expected.title(), entry.title());
            });

        let item_list_for_search =
            vault.get_item_list_for(&[42_u8; uuid::SIZE].into(), Some("Group"));
        assert_eq!(item_list_for_search.groups.len(), groups.len());
        assert!(item_list_for_search.entries.is_empty());
        groups
            .iter()
            .zip(item_list_for_search.groups.iter())
            .for_each(|(expected, group)| {
                assert_eq!(expected.title(), group.title());
            });

        let item_list_for_search =
            vault.get_item_list_for(&[42_u8; uuid::SIZE].into(), Some("Entry"));
        assert!(item_list_for_search.groups.is_empty());
        assert_eq!(item_list_for_search.entries.len(), entries.len());
        entries
            .iter()
            .zip(item_list_for_search.entries.iter())
            .for_each(|(expected, entry)| {
                assert_eq!(expected.title(), entry.title());
            });

        let item_list_for_search = vault.get_item_list_for(&[42_u8; uuid::SIZE].into(), Some("5"));
        assert_eq!(item_list_for_search.groups.len(), 2);
        assert_eq!(item_list_for_search.entries.len(), 2);
        assert!(item_list_for_search
            .groups
            .iter()
            .all(|group| group.title().contains("5")));
        assert!(item_list_for_search
            .entries
            .iter()
            .all(|entry| entry.title().contains("5")));
    }

    #[test]
    fn item_list_is_empty() {
        let item_list = ItemList {
            groups: vec![],
            entries: vec![],
        };

        assert!(item_list.is_empty());

        let group = Group::new(
            [42_u8; uuid::SIZE].into(),
            [21_u8; uuid::SIZE].into(),
            "Title".into(),
        );

        let item_list = ItemList {
            groups: vec![&group],
            entries: vec![],
        };

        assert!(!item_list.is_empty());

        let entry = EntryHead::new(
            [42_u8; uuid::SIZE].into(),
            [21_u8; uuid::SIZE].into(),
            "title".into(),
            [84_u8; uuid::SIZE].into(),
        );

        let item_list = ItemList {
            groups: vec![],
            entries: vec![&entry],
        };

        assert!(!item_list.is_empty());

        let item_list = ItemList {
            groups: vec![&group],
            entries: vec![&entry],
        };

        assert!(!item_list.is_empty());
    }
}
