//! TODO
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
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The masterkey to encrypt the group
    pub fn save(&mut self, path: &Path, masterkey: &[u8]) -> Result<(), PWDuckCoreError> {
        let group = self.encrypt(masterkey)?;
        crate::io::save_group(path, &self.uuid, &group)?;
        self.modified = false;
        Ok(())
    }

    /// Encrypt this [`Group`](Group) with the given masterkey.
    fn encrypt(&self, masterkey: &[u8]) -> Result<crate::dto::group::Group, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(content.as_bytes(), masterkey, &iv)?;
        content.zeroize();
        Ok(crate::dto::group::Group::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// Load a [`Group`](Group) from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The UUID as the identifier of the [`Group`](Group)
    ///     - The masterkey to decrypt the [`Group`](Group)
    pub fn load(path: &Path, uuid: &Uuid, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_group(path, uuid)?;
        Self::decrypt(&dto, masterkey)
    }

    /// Load all [`Group`](Group)s from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The masterkey to decrypt the [`Group`](Group)s
    pub fn load_all(path: &Path, masterkey: &[u8]) -> Result<HashMap<Uuid, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_groups(path)?;

        let mut results = HashMap::new();

        for dto in dtos {
            let group = Self::decrypt(&dto, masterkey)?;
            drop(results.insert(group.uuid().clone(), group));
        }

        Ok(results)
    }

    /// Decrypt the data-transfer-object (dto) of the [`Group`](Group) with the given masterkey.
    fn decrypt(dto: &crate::dto::group::Group, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            masterkey,
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
