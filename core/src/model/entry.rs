//! TODO
use std::{collections::HashMap, convert::TryInto, ops::Deref, path::Path};

use crate::{
    cryptography::{self, aes_cbc_decrypt, aes_cbc_encrypt, generate_aes_iv},
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
    parent: Uuid,

    /// The title of this entry.
    #[getset(get = "pub")]
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
    body: Uuid,

    /// If the head was modified.
    #[serde(skip)]
    modified: bool,
}

impl EntryHead {
    /// Create a new [`EntryHead`](EntryHead).
    #[must_use]
    pub fn new(uuid: Uuid, parent: Uuid, title: String, body: Uuid) -> Self {
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
        crate::io::save_entry_head(path, &self.uuid, &entry_head)?;
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
    pub fn load(path: &Path, uuid: &Uuid, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_head(path, uuid)?;
        Self::decrypt(&dto, masterkey)
    }

    /// Load all [`EntryHead`](EntryHead)s from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///     - The masterkey to decrypt the [`EntryHead`](EntryHead)s
    pub fn load_all(path: &Path, masterkey: &[u8]) -> Result<HashMap<Uuid, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_entry_heads(path)?;

        let mut results = HashMap::new();

        for dto in dtos {
            let head = Self::decrypt(&dto, masterkey)?;
            drop(results.insert(head.uuid().clone(), head));
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

    /// Set the title of this entry.
    pub fn set_title(&mut self, title: String) -> &mut Self {
        self.title.zeroize();
        self.title = title;
        self.modified = true;
        self
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
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
#[derive(Getters, Setters)]
pub struct EntryBody {
    /// The UUID of this body.
    #[getset(get = "pub")]
    uuid: Uuid,

    /// The username of this entry.
    #[getset(get = "pub")]
    username: SecString,

    /// The password of this entry.
    #[getset(get = "pub")]
    password: SecString,

    /// The email of this entry.
    #[getset(get = "pub")]
    email: SecString,

    /// If the body was modified.
    modified: bool,
}

impl EntryBody {
    /// Create a new [`EntryBody`](EntryBody).
    #[must_use]
    pub fn new(uuid: Uuid, username: String, password: String) -> Self {
        Self {
            uuid,
            username: username.into(),
            password: password.into(),
            email: SecString::new(),
            modified: true,
        }
    }

    /// Encrypt this [`EntryBody`](EntryBody) with the given masterkey.
    pub fn encrypt(
        &self,
        master_key: &[u8],
    ) -> Result<crate::dto::entry::EntryBody, PWDuckCoreError> {
        let iv = generate_aes_iv();

        let encrypted_body = EncryptedBody::from(self, master_key)?;

        let mut content = ron::to_string(&encrypted_body)?;
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
    pub fn load(path: &Path, uuid: &Uuid, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
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

        let encrypted_body: EncryptedBody = ron::from_str(&content)?;

        let body = encrypted_body.into(masterkey)?;

        Ok(body)
    }

    /// Set the username of this entry.
    pub fn set_username(&mut self, username: String) -> &mut Self {
        self.username = username.into();
        self.modified = true;
        self
    }

    /// Set the password of this entry.
    pub fn set_password(&mut self, password: String) -> &mut Self {
        self.password = password.into();
        self.modified = true;
        self
    }

    /// Set the email of this entry.
    pub fn set_email(&mut self, email: String) -> &mut Self {
        self.email = email.into();
        self.modified = true;
        self
    }

    /// True, if the [`EntryBody`](EntryBody) was modified.
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }
}

/// The encrypted data of an [`EntryBody`](EntryBody).
#[derive(Deserialize, Serialize)]
struct EncryptedBody {
    /// The iv used for the encryption.
    iv: Vec<u8>,
    /// The encrypted UUID of this entry.
    uuid: Vec<u8>,
    /// The encrypted username of this entry.
    username: Vec<u8>,
    /// The encrypted password of this entry.
    password: Vec<u8>,
    /// The encrypted email of this entry.
    #[serde(default)]
    email: Vec<u8>,
}

impl EncryptedBody {
    /// Encrypt the given [`EntryBody`](EntryBody) with the masterkey.
    fn from(body: &EntryBody, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let iv = cryptography::generate_aes_iv();
        Ok(Self {
            iv: iv.clone(),
            uuid: aes_cbc_encrypt(&body.uuid, masterkey, &iv)?,
            username: aes_cbc_encrypt(body.username.as_bytes(), masterkey, &iv)?,
            password: aes_cbc_encrypt(body.password.as_bytes(), masterkey, &iv)?,
            email: aes_cbc_encrypt(body.email.as_bytes(), masterkey, &iv)?,
        })
    }

    /// Decrypt the [`EncryptedBody`](EncryptedBody) with the masterkey.
    fn into(self, masterkey: &[u8]) -> Result<EntryBody, PWDuckCoreError> {
        Ok(EntryBody {
            uuid: aes_cbc_decrypt(&self.uuid, masterkey, &self.iv)?.try_into()?,
            username: SecString::from_utf8(aes_cbc_decrypt(&self.username, masterkey, &self.iv)?)?,
            password: SecString::from_utf8(aes_cbc_decrypt(&self.password, masterkey, &self.iv)?)?,
            email: SecString::from_utf8(aes_cbc_decrypt(&self.email, masterkey, &self.iv)?)?,
            modified: false,
        })
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
