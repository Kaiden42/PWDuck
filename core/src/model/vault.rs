//! TODO

use std::{collections::HashMap, path::PathBuf};

use zeroize::Zeroize;

use crate::{
    cryptography::{
        decrypt_masterkey, derive_key_protection, generate_argon2_salt, generate_chacha20_nonce,
        generate_masterkey, unprotect_masterkey,
    },
    error::PWDuckCoreError,
    io::{create_new_vault_dir, save_masterkey},
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
    masterkey: MasterKey,

    /// The salt to derive the key to decrypt the in-memory encrypted masterkey.
    #[getset(get = "pub")]
    salt: String,

    /// The nonce used to decrypt the in-memory encrypted masterkey.
    #[getset(get = "pub")]
    nonce: Vec<u8>,

    /// The [`PathBuf`](PathBuf) of the location of this [`Vault`](Vault).
    #[getset(get = "pub")]
    path: PathBuf,

    /// The [`Group`](Group)s of this [`Vault`](Vault).
    #[getset(get = "pub", get_mut = "pub")]
    groups: HashMap<String, Group>,

    /// The [`EntryHead`](EntryHead)s of this vault.
    #[getset(get = "pub")]
    entries: HashMap<String, EntryHead>,

    /// The encrypted data-transfer-objects (dtos) of unsaved [`EntryBody`](EntryBody)s.
    #[getset(get = "pub")]
    unsaved_entry_bodies: HashMap<String, crate::dto::entry::EntryBody>,

    /// TODO
    deleted_groups: Vec<String>,
    /// TODO: (head, body)
    deleted_entries: Vec<(String, String)>,
}

impl Vault {
    /// Generate a new [`Vault`](Vault).
    ///
    /// It expects:
    ///     - The password to encrypt the masterkey of the new [`Vault`](Vault)
    ///     - The memory key to protect the new generated masterkey of the new [`Vault`](Vault)
    ///     - The path as the location of the new [`Vault`](Vault)
    pub fn generate<P>(password: &str, mem_key: &MemKey, path: P) -> Result<Self, PWDuckCoreError>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        create_new_vault_dir(&path)?;

        let masterkey_dto = generate_masterkey(password)?;

        let salt = generate_argon2_salt();
        let nonce = generate_chacha20_nonce();

        let masterkey = decrypt_masterkey(
            &masterkey_dto,
            password,
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;

        let mut vault = Self {
            masterkey,
            salt,
            nonce,
            path,
            groups: HashMap::new(),
            entries: HashMap::new(),
            unsaved_entry_bodies: HashMap::new(),
            deleted_groups: Vec::new(),
            deleted_entries: Vec::new(),
        };

        let root = Group::create_root_for(vault.path());
        drop(vault.groups_mut().insert(root.uuid().as_string(), root));

        save_masterkey(vault.path(), masterkey_dto)?;
        vault.save(mem_key)?;

        Ok(vault)
    }

    /// Save the vault to disk.
    ///
    /// It expects:
    ///     - The [`MemKey`](MemKey) to decrypt the in-memory encrypted masterkey of the [`Vault`](Vault).
    pub fn save(&mut self, mem_key: &MemKey) -> Result<(), PWDuckCoreError> {
        let path = self.path.clone();
        let mut masterkey = unprotect_masterkey(
            self.masterkey.key(),
            &derive_key_protection(mem_key, &self.salt)?,
            &self.nonce,
        )?;

        let unsaved_entry_bodies_result: Result<(), PWDuckCoreError> = self
            .unsaved_entry_bodies
            .iter()
            .try_for_each(|(uuid, entry_body)| crate::io::save_entry_body(&path, uuid, entry_body));
        if unsaved_entry_bodies_result.is_ok() {
            self.unsaved_entry_bodies.clear()
        }

        let group_result: Result<(), PWDuckCoreError> = self
            .groups
            .iter_mut()
            .filter(|(_, group)| group.is_modified())
            .try_for_each(|(_, group)| group.save(&path, &masterkey));

        let entry_result: Result<(), PWDuckCoreError> = self
            .entries
            .iter_mut()
            .filter(|(_, entry)| entry.is_modified())
            .try_for_each(|(_, entry)| entry.save(&path, &masterkey));

        masterkey.zeroize();

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
    ///     - The password to decrypt the masterkey of the [`Vault`](Vault)
    ///     - The [`MemKey`] to re-encrypt the decrypted masterkey in memory
    ///     - The path as the location of the vault
    pub fn load<P>(password: &str, mem_key: &MemKey, path: P) -> Result<Self, PWDuckCoreError>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        let salt = generate_argon2_salt();
        let nonce = generate_chacha20_nonce();

        //let masterkey = decrypt_masterkey(&masterkey_dto, password, &derive_key_protection(mem_key, salt.as_str())?, &nonce)?;
        let masterkey = MasterKey::load(
            &path,
            password,
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;

        let unprotected_masterkey = unprotect_masterkey(
            masterkey.key(),
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;
        let groups = Group::load_all(&path, &unprotected_masterkey)?;
        let entries = EntryHead::load_all(&path, &unprotected_masterkey)?;
        drop(unprotected_masterkey);

        let vault = Self {
            masterkey,
            salt,
            nonce,
            path,
            groups,
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
    pub fn get_root_uuid(&self) -> Option<String> {
        //println!("groups: {:?}", self.get_groups());
        //self.groups.get("").map(|r| r.get_uuid().as_string())
        self.groups
            .iter()
            .find(|(_uuid, group)| group.is_root())
            .map(|(_uuid, group)| group.uuid().as_string())
    }

    /// Insert a new [`Group`](Group) into this [`Vault`](Vault).
    pub fn insert_group(&mut self, group: Group) {
        drop(self.groups.insert(group.uuid().as_string(), group));
    }

    /// TODO
    pub fn delete_group(&mut self, uuid: &Uuid) {
        let uuid = uuid.as_string();
        if let Some(_group) = self.groups.remove(&uuid) {
            self.deleted_groups.push(uuid);
        }
    }

    /// Insert a new entry into this [`Vault`](Vault).
    ///
    /// It expects:
    ///     - The [`EntryHead`] of the new entry
    ///     - The [`EntryBody`] of the new entry
    ///     - The masterkey to decrypt the [`EntryBody`](EntryBody)
    pub fn insert_entry(
        &mut self,
        entry_head: EntryHead,
        entry_body: EntryBody,
        masterkey: &[u8],
    ) -> Result<(), PWDuckCoreError> {
        drop(
            self.entries
                .insert(entry_head.uuid().as_string(), entry_head),
        );

        drop(self.unsaved_entry_bodies.insert(
            entry_body.uuid().as_string(),
            entry_body.encrypt(masterkey)?,
        ));
        drop(entry_body);
        Ok(())
    }

    /// TODO
    pub fn delete_entry(&mut self, uuid: &Uuid) {
        let uuid = uuid.as_string();
        if let Some(entry_head) = self.entries.remove(&uuid) {
            let entry_body = entry_head.body();
            self.deleted_entries.push((uuid, entry_body.clone()));
        }
    }

    /// Get all [`Group`](Group)s in this [`Vault`] that are the children of the specified parent [`Group`](Group).
    #[must_use]
    pub fn get_groups_of(&self, parent_uuid: &str) -> Vec<&Group> {
        self.groups
            .iter()
            .filter(|(_uuid, group)| group.parent() == parent_uuid)
            .map(|(_uuid, group)| group)
            .collect()
    }

    /// Get all [`EntryHead`](EntryHead)s in this [`Vault`] that are children of the specified parent [`Group`](Group).
    #[must_use]
    pub fn get_entries_of(&self, parent_uuid: &str) -> Vec<&EntryHead> {
        self.entries
            .iter()
            .filter(|(_uuid, entry)| entry.parent() == parent_uuid)
            .map(|(_uuid, entry)| entry)
            .collect()
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
    ///     - The UUID of the current selected [`Group`](Group)
    ///     - The optional search filter
    #[must_use]
    pub fn get_item_list_for<'a>(
        &'a self,
        selected_group_uuid: &str,
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
    use std::path::PathBuf;

    use crate::mem_protection;

    use super::Vault;

    #[test]
    fn test_generate_new_vault() {
        let mem_key = mem_protection::MemKey::new();
        let path: PathBuf = "this_is_a_test_vault".into();

        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }

        let vault = Vault::generate("this is a pretty cool password", &mem_key, &path)
            .expect("Vault generation should not fail");

        // TODO
        std::fs::remove_dir_all(&path).unwrap();
    }
}
