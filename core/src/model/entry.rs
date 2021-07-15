//! TODO
use std::{collections::HashMap, ops::Deref, path::Path};

use crate::{
    cryptography::{aes_cbc_decrypt, aes_cbc_encrypt, generate_aes_iv},
    error::PWDuckCoreError,
    mem_protection::SecString,
};
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use super::uuid::Uuid;
/// The in-memory representation of an entry head.
#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
#[derive(Getters, Setters)]
pub struct EntryHead {
    /// The UUID of this head.
    #[getset(get = "pub")]
    uuid: Uuid,

    /// The UUID of the parent [`Group`](crate::model::group::Group) of this head.
    #[getset(get = "pub")]
    parent: String,

    /// The title of this entry.
    #[getset(get = "pub", set = "pub")]
    title: String,

    /// The address of the website this entry belongs to.
    #[getset(get = "pub")]
    #[serde(default)]
    web_address: String,

    /// TODO
    #[getset(get = "pub", set = "pub")]
    #[serde(default)]
    auto_type_sequence: AutoTypeSequence,

    /// The UUID of the body of this entry.
    #[getset(get = "pub")]
    body: String,

    /// If the head was modified.
    #[serde(skip)]
    modified: bool,
}

impl EntryHead {
    /// Create a new [`EntryHead`](EntryHead).
    #[must_use]
    pub fn new(uuid: Uuid, parent: String, title: String, body: String) -> Self {
        Self {
            uuid,
            parent,
            title,
            web_address: String::new(),
            auto_type_sequence: AutoTypeSequence::default(),
            body,
            modified: true,
        }
    }

    /// Save the [`EntryHead`] to disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault](crate::Vault).
    ///     - The masterkey to encrypt the head.
    pub fn save(&mut self, path: &Path, masterkey: &[u8]) -> Result<(), PWDuckCoreError> {
        let entry_head = self.encrypt(masterkey)?;
        crate::io::save_entry_head(path, &self.uuid.as_string(), &entry_head)?;
        self.modified = false;
        Ok(())
    }

    /// Encrypt this [`EntryHead`](EntryHead) with the given masterkey.
    fn encrypt(&self, masterkey: &[u8]) -> Result<crate::dto::entry::EntryHead, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(content.as_bytes(), masterkey, &iv)?;
        content.zeroize();
        Ok(crate::dto::entry::EntryHead::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// Load an [`EntryHead`](EntryHead) from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The UUID as the identifier of the [`EntryHead`](EntryHead)
    ///     - The masterkey to decrypt the [`EntryHead`](EntryHead)
    pub fn load(path: &Path, uuid: &str, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_head(path, uuid)?;
        Self::decrypt(&dto, masterkey)
    }

    /// Load all [`EntryHead`](EntryHead)s from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The masterkey to decrypt the [`EntryHead`](EntryHead)s
    pub fn load_all(
        path: &Path,
        masterkey: &[u8],
    ) -> Result<HashMap<String, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_entry_heads(path)?;

        //let mut results = Vec::with_capacity(dtos.len());
        let mut results = HashMap::new();

        for dto in dtos {
            //results.push(Self::decrypt(dto, masterkey)?);
            let head = Self::decrypt(&dto, masterkey)?;
            drop(results.insert(head.uuid().as_string(), head));
        }

        Ok(results)
    }

    /// Decrypt the data-transfer-object (dto) of the [`EntryHead`] with the given masterkey.
    fn decrypt(
        dto: &crate::dto::entry::EntryHead,
        masterkey: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            masterkey,
            &base64::decode(dto.iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;
        let head = ron::from_str(&content)?;

        Ok(head)
    }

    /// Set the web address of this entry.
    pub fn set_web_address(&mut self, web_address: String) -> &mut Self {
        self.web_address.zeroize();
        self.web_address = web_address;
        self.modified = true;
        self
    }

    /// True, if the [`EntryHead`](EntryHead) was modified.
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }
}

/// The in-memory representation of an entry body.
#[allow(missing_debug_implementations)]
#[derive(Clone, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
#[derive(Getters, Setters)]
pub struct EntryBody {
    /// The UUID of this body.
    #[getset(get = "pub")]
    uuid: Uuid,

    /// The username of this entry.
    #[getset(get = "pub")]
    username: String,

    /// The password of this entry.
    #[getset(get = "pub")]
    password: String,

    /// The email of this entry.
    #[getset(get = "pub")]
    #[serde(default)]
    email: String,

    /// If the body was modified.
    #[serde(skip)]
    modified: bool,
}

impl EntryBody {
    /// Create a new [`EntryBody`](EntryBody).
    #[must_use]
    pub const fn new(uuid: Uuid, username: String, password: String) -> Self {
        Self {
            uuid,
            username,
            password,
            email: String::new(),
            modified: true,
        }
    }

    /// Encrypt this [`EntryBody`](EntryBody) with the given masterkey.
    pub fn encrypt(
        &self,
        master_key: &[u8],
    ) -> Result<crate::dto::entry::EntryBody, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(content.as_bytes(), master_key, &iv)?;
        content.zeroize();
        Ok(crate::dto::entry::EntryBody::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// Load an [`EntryBody`](EntryBody) from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The UUID as the identifier of the [`EntryBody`](EntryBody)
    ///     - The masterkey to decrypt the [`EntryBody`](EntryBody)
    pub fn load(path: &Path, uuid: &str, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_body(path, uuid)?;
        let body = Self::decrypt(&dto, masterkey)?;
        Ok(body)
    }

    /// Decrypt the data-transfer-object (dto) of the [`EntryBody`](EntryBody) with the given masterkey.
    pub fn decrypt(
        dto: &crate::dto::entry::EntryBody,
        masterkey: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            masterkey,
            &base64::decode(dto.iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;
        let body = ron::from_str(&content)?;

        Ok(body)
    }

    /// Set the username of this entry.
    pub fn set_username(&mut self, username: String) -> &mut Self {
        self.username.zeroize();
        self.username = username;
        self.modified = true;
        self
    }

    /// Set the password of this entry.
    pub fn set_password(&mut self, password: String) -> &mut Self {
        self.password.zeroize();
        self.password = password;
        self.modified = true;
        self
    }

    /// Set the email of this entry.
    pub fn set_email(&mut self, email: String) -> &mut Self {
        self.email.zeroize();
        self.email = email;
        self.modified = true;
        self
    }

    /// True, if the [`EntryBody`](EntryBody) was modified.
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }
}

/// The sequence to specify how to autotype.
#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
pub struct AutoTypeSequence {
    /// The autotype sequence.
    sequence: String,
}

impl Default for AutoTypeSequence {
    fn default() -> Self {
        Self {
            sequence: "[username]<tab>[password]<enter>".into(),
        }
    }
}

impl Deref for AutoTypeSequence {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.sequence
    }
}

impl From<String> for AutoTypeSequence {
    fn from(string: String) -> Self {
        Self { sequence: string }
    }
}
