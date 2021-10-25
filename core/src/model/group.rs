//! Decrypted groups stored in memory.
use std::{collections::HashMap, path::Path};

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{
    cryptography::{aes_cbc_decrypt, aes_cbc_encrypt, generate_aes_iv},
    error::PWDuckCoreError,
    mem_protection::SecString,
};

use super::uuid::Uuid;

/// The in-memory representation of a group.
#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
#[derive(Getters, Setters)]
pub struct Group {
    /// The UUID of this group.
    #[getset(get = "pub")]
    uuid: Uuid,

    /// The UUID of the parent of this group.
    #[getset(get = "pub")]
    parent: Option<Uuid>,

    /// The title of this group.
    #[getset(get = "pub")]
    title: String,

    /// If the group was modified.
    #[serde(skip)]
    modified: bool,
}

impl Group {
    /// Create an new [`Group`](Group).
    #[must_use]
    pub const fn new(uuid: Uuid, parent: Uuid, title: String) -> Self {
        Self {
            uuid,
            parent: Some(parent),
            title,
            modified: true,
        }
    }

    /// Save the [`Group`](Group) to disk.
    ///
    /// It expects:
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///  - The masterkey to encrypt the group
    pub fn save(&mut self, path: &Path, master_key: &[u8]) -> Result<(), PWDuckCoreError> {
        let group = self.encrypt(master_key)?;
        crate::io::save_group(path, &self.uuid, &group)?;
        self.modified = false;
        Ok(())
    }

    /// Encrypt this [`Group`](Group) with the given masterkey.
    fn encrypt(&self, master_key: &[u8]) -> Result<crate::dto::group::Group, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(content.as_bytes(), master_key, &iv)?;
        content.zeroize();
        Ok(crate::dto::group::Group::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// Load a [`Group`](Group) from disk.
    ///
    /// It expects:
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///  - The UUID as the identifier of the [`Group`](Group)
    ///  - The masterkey to decrypt the [`Group`](Group)
    pub fn load(path: &Path, uuid: &Uuid, master_key: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_group(path, uuid)?;
        Self::decrypt(&dto, master_key)
    }

    /// Load all [`Group`](Group)s from disk.
    ///
    /// It expects:
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///  - The masterkey to decrypt the [`Group`](Group)s
    pub fn load_all(path: &Path, master_key: &[u8]) -> Result<HashMap<Uuid, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_groups(path)?;

        let mut results = HashMap::new();

        for dto in dtos {
            let group = Self::decrypt(&dto, master_key)?;
            drop(results.insert(group.uuid().clone(), group));
        }

        Ok(results)
    }

    /// Decrypt the data-transfer-object (dto) of the [`Group`](Group) with the given masterkey.
    fn decrypt(dto: &crate::dto::group::Group, master_key: &[u8]) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            master_key,
            &base64::decode(dto.iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;
        let group = ron::from_str(&content)?;

        Ok(group)
    }

    /// Create a new root group on the given path.
    #[must_use]
    pub fn create_root_for(path: &Path) -> Self {
        Self {
            uuid: Uuid::new(path),
            parent: None,
            title: String::new(),
            modified: true,
        }
    }

    /// True, if this group is the root.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    /// Set the title of this group.
    pub fn set_title(&mut self, title: String) -> &mut Self {
        self.title = title;
        self.modified = true;
        self
    }

    /// True, if this group was modified.
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mocktopus::mocking::*;
    use tempfile::tempdir;

    use crate::{cryptography, io::create_new_vault_dir, model::uuid, Uuid};

    use super::Group;

    use lazy_static::lazy_static;
    lazy_static! {
        static ref DEFAULT_GROUP_UUID: Uuid = [42_u8; uuid::SIZE].into();
        static ref DEFAULT_PARENT_UUID: Uuid = [21_u8; uuid::SIZE].into();
        static ref DEFAULT_GROUP: Group = Group::new(
            DEFAULT_GROUP_UUID.to_owned(),
            DEFAULT_PARENT_UUID.to_owned(),
            "Default title".into(),
        );
    }

    #[test]
    fn new_group() {
        let uuid: Uuid = [42_u8; uuid::SIZE].into();
        let parent: Uuid = [21_u8; uuid::SIZE].into();
        let title = "Title";

        let group = Group::new(uuid.clone(), parent.clone(), title.to_owned());

        assert_eq!(group.uuid, uuid);
        assert_eq!(group.parent, Some(parent));
        assert_eq!(group.title.as_str(), title);
        assert!(group.modified)
    }

    #[test]
    fn encrypt_and_decrypt_head() {
        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let group = DEFAULT_GROUP.to_owned();

        let encrypted: crate::dto::group::Group = group
            .encrypt(&master_key)
            .expect("Encrypting group should not fail.");

        let decrypted =
            Group::decrypt(&encrypted, &master_key).expect("Decrypting group should not fail.");

        assert!(equal_groups(&group, &decrypted));
    }

    #[test]
    fn save_and_load_group() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let mut group = DEFAULT_GROUP.to_owned();
        assert!(group.modified);

        group
            .save(&path, &master_key)
            .expect("Saving group should not fail.");
        assert!(!group.modified);

        let loaded =
            Group::load(&path, group.uuid(), &master_key).expect("Loading group should not fail.");

        assert!(equal_groups(&group, &loaded));
    }

    #[test]
    fn load_all_groups() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let groups: HashMap<Uuid, Group> =
            (0..=10).into_iter().fold(HashMap::new(), |mut m, next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let mut group = Group::new(
                    uuid.clone(),
                    [255_u8; uuid::SIZE].into(),
                    format!("Group: {}", next),
                );

                group.save(&path, &master_key).unwrap();

                drop(m.insert(uuid, group));
                m
            });

        let loaded =
            Group::load_all(&path, &master_key).expect("Loading all groups should not fail.");

        assert_eq!(groups.len(), loaded.len());
        for (uuid, group) in groups {
            let load = loaded
                .get(&uuid)
                .expect("Loaded should contain this group.");
            assert!(equal_groups(&group, &load));
        }
    }

    #[test]
    fn create_root() {
        crate::cryptography::fill_random_bytes.mock_safe(|buf| {
            buf.fill(42_u8);
            MockResult::Return(())
        });

        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let root = Group::create_root_for(&path);

        assert_eq!(root.uuid, [42_u8; uuid::SIZE].into());
        assert_eq!(root.parent, None);
        assert_eq!(root.title.as_str(), "");
        assert!(root.modified);

        assert!(root.is_root());
        assert!(!DEFAULT_GROUP.is_root());
    }

    #[test]
    fn set_title() {
        let title = "Custom title";

        let mut group = DEFAULT_GROUP.to_owned();
        group.modified = false;

        assert_eq!(group.title, String::from("Default title"));

        let _ = group.set_title(title.to_owned());

        assert!(group.modified);
        assert_eq!(group.title.as_str(), title);
    }

    #[test]
    fn is_modified() {
        let mut group = DEFAULT_GROUP.to_owned();

        assert!(group.is_modified());
        group.modified = false;
        assert!(!group.is_modified());
    }

    fn equal_groups(a: &Group, b: &Group) -> bool {
        a.uuid == b.uuid && a.parent == b.parent && a.title == b.title
    }
}
