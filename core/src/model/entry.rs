//! Decrypted entries stored in memory.
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

    /// The sequence of the auto type.
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
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault).
    ///  - The master key to encrypt the head.
    pub fn save(&mut self, path: &Path, master_key: &[u8]) -> Result<(), PWDuckCoreError> {
        let entry_head = self.encrypt(master_key)?;
        crate::io::save_entry_head(path, &self.uuid, &entry_head)?;
        self.modified = false;
        Ok(())
    }

    /// Encrypt this [`EntryHead`](EntryHead) with the given master key.
    fn encrypt(&self, master_key: &[u8]) -> Result<crate::dto::entry::EntryHead, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(content.as_bytes(), master_key, &iv)?;
        content.zeroize();
        Ok(crate::dto::entry::EntryHead::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// Load an [`EntryHead`](EntryHead) from disk.
    ///
    /// It expects:
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///  - The UUID as the identifier of the [`EntryHead`](EntryHead)
    ///  - The master key to decrypt the [`EntryHead`](EntryHead)
    pub fn load(path: &Path, uuid: &Uuid, master_key: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_head(path, uuid)?;
        Self::decrypt(&dto, master_key)
    }

    /// Load all [`EntryHead`](EntryHead)s from disk.
    ///
    /// It expects:
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///  - The master key to decrypt the [`EntryHead`](EntryHead)s
    pub fn load_all(path: &Path, master_key: &[u8]) -> Result<HashMap<Uuid, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_entry_heads(path)?;

        let mut results = HashMap::new();

        for dto in dtos {
            let head = Self::decrypt(&dto, master_key)?;
            drop(results.insert(head.uuid().clone(), head));
        }

        Ok(results)
    }

    /// Decrypt the data-transfer-object (dto) of the [`EntryHead`] with the given master key.
    fn decrypt(
        dto: &crate::dto::entry::EntryHead,
        master_key: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            master_key,
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

    /// Encrypt this [`EntryBody`](EntryBody) with the given master key.
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
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault)
    ///  - The UUID as the identifier of the [`EntryBody`](EntryBody)
    ///  - The master key to decrypt the [`EntryBody`](EntryBody)
    pub fn load(path: &Path, uuid: &Uuid, master_key: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_body(path, uuid)?;
        let body = Self::decrypt(&dto, master_key)?;
        Ok(body)
    }

    /// Decrypt the data-transfer-object (dto) of the [`EntryBody`](EntryBody) with the given master key.
    pub fn decrypt(
        dto: &crate::dto::entry::EntryBody,
        master_key: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            master_key,
            &base64::decode(dto.iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;

        let encrypted_body: EncryptedBody = ron::from_str(&content)?;

        let body = encrypted_body.into(master_key)?;

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
    /// Encrypt the given [`EntryBody`](EntryBody) with the master key.
    fn from(body: &EntryBody, master_key: &[u8]) -> Result<Self, PWDuckCoreError> {
        let iv = cryptography::generate_aes_iv();
        Ok(Self {
            iv: iv.clone(),
            uuid: aes_cbc_encrypt(&body.uuid, master_key, &iv)?,
            username: aes_cbc_encrypt(body.username.as_bytes(), master_key, &iv)?,
            password: aes_cbc_encrypt(body.password.as_bytes(), master_key, &iv)?,
            email: aes_cbc_encrypt(body.email.as_bytes(), master_key, &iv)?,
        })
    }

    /// Decrypt the [`EncryptedBody`](EncryptedBody) with the master key.
    fn into(self, master_key: &[u8]) -> Result<EntryBody, PWDuckCoreError> {
        Ok(EntryBody {
            uuid: aes_cbc_decrypt(&self.uuid, master_key, &self.iv)?.try_into()?,
            username: SecString::from_utf8(aes_cbc_decrypt(&self.username, master_key, &self.iv)?)?,
            password: SecString::from_utf8(aes_cbc_decrypt(&self.password, master_key, &self.iv)?)?,
            email: SecString::from_utf8(aes_cbc_decrypt(&self.email, master_key, &self.iv)?)?,
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

/// The default auto type sequence.
const DEFAULT_SEQUENCE: &str = "[username]<tab>[password]<enter>";
impl Default for AutoTypeSequence {
    fn default() -> Self {
        Self {
            sequence: DEFAULT_SEQUENCE.to_owned(),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mocktopus::mocking::*;
    use tempfile::tempdir;

    use crate::{cryptography, io::create_new_vault_dir, model::uuid, SecString, Uuid};

    use super::{AutoTypeSequence, EncryptedBody, EntryBody, EntryHead, DEFAULT_SEQUENCE};

    use lazy_static::lazy_static;
    lazy_static! {
        static ref DEFAULT_HEAD_UUID: Uuid = [42_u8; uuid::SIZE].into();
        static ref DEFAULT_PARENT_UUID: Uuid = [21_u8; uuid::SIZE].into();
        static ref DEFAULT_BODY_UUID: Uuid = [84_u8; uuid::SIZE].into();
        static ref DEFAULT_HEAD: EntryHead = EntryHead::new(
            DEFAULT_HEAD_UUID.to_owned(),
            DEFAULT_PARENT_UUID.to_owned(),
            "Default title".into(),
            DEFAULT_BODY_UUID.to_owned(),
        );
        static ref DEFAULT_BODY: EntryBody = EntryBody::new(
            DEFAULT_BODY_UUID.to_owned(),
            "Default username".into(),
            "Default password".into(),
        );
    }

    #[test]
    fn new_entry_head() {
        let uuid: Uuid = [42_u8; uuid::SIZE].into();
        let parent: Uuid = [21_u8; uuid::SIZE].into();
        let body: Uuid = [84_u8; uuid::SIZE].into();
        let title = "Title";

        let head = EntryHead::new(uuid.clone(), parent.clone(), title.to_owned(), body.clone());

        assert_eq!(head.uuid, uuid);
        assert_eq!(head.parent, parent);
        assert_eq!(head.title.as_str(), title);
        assert_eq!(head.web_address, String::from(""));
        assert_eq!(
            head.auto_type_sequence.sequence,
            AutoTypeSequence::default().sequence
        );
        assert_eq!(head.body, body);
        assert!(head.modified);
    }

    #[test]
    fn encrypt_and_decrypt_head() {
        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let head = DEFAULT_HEAD.to_owned();

        let encrypted: crate::dto::entry::EntryHead = head
            .encrypt(&master_key)
            .expect("Encrypting entry head should not fail.");

        let decrypted = EntryHead::decrypt(&encrypted, &master_key)
            .expect("Decrypting entry head should not fail.");

        assert!(equal_heads(&head, &decrypted));
    }

    #[test]
    fn save_and_load_head() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let mut head = DEFAULT_HEAD.to_owned();
        assert!(head.modified);

        head.save(&path, &master_key)
            .expect("Saving entry head should not fail.");
        assert!(!head.modified);

        let loaded = EntryHead::load(&path, head.uuid(), &master_key)
            .expect("Loading entry head should not fail.");

        assert!(equal_heads(&head, &loaded));
    }

    #[test]
    fn load_all_heads() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let heads: HashMap<Uuid, EntryHead> =
            (0..=10).into_iter().fold(HashMap::new(), |mut m, next| {
                let uuid: Uuid = [next; uuid::SIZE].into();
                let mut head = EntryHead::new(
                    uuid.clone(),
                    [255_u8; uuid::SIZE].into(),
                    format!("Head: {}", next),
                    [254_u8; uuid::SIZE].into(),
                );

                head.save(&path, &master_key).unwrap();

                drop(m.insert(uuid, head));
                m
            });

        let loaded = EntryHead::load_all(&path, &master_key)
            .expect("Loading all entry heads should not fail.");

        assert_eq!(heads.len(), loaded.len());
        for (uuid, head) in heads {
            let load = loaded.get(&uuid).expect("Loaded should contain this head.");
            assert!(equal_heads(&head, &load));
        }
    }

    #[test]
    fn set_title() {
        let title = "Custom title";

        let mut head = DEFAULT_HEAD.to_owned();
        head.modified = false;

        assert_eq!(head.title, String::from("Default title"));

        let _ = head.set_title(title.to_owned());

        assert!(head.modified);
        assert_eq!(head.title.as_str(), title);
    }

    #[test]
    fn set_web_address() {
        let web_address = "https://example.web";

        let mut head = DEFAULT_HEAD.to_owned();
        head.modified = false;

        assert_eq!(head.web_address, String::new());

        let _ = head.set_web_address(web_address.to_owned());

        assert!(head.modified);
        assert_eq!(head.web_address.as_str(), web_address);
    }

    #[test]
    fn is_modified_head() {
        let mut head = DEFAULT_HEAD.to_owned();

        assert!(head.is_modified());
        head.modified = false;
        assert!(!head.is_modified());
    }

    fn equal_heads(a: &EntryHead, b: &EntryHead) -> bool {
        a.uuid == b.uuid
            && a.parent == b.parent
            && a.title == b.title
            && a.web_address == b.web_address
            && a.auto_type_sequence.sequence == b.auto_type_sequence.sequence
            && a.body == b.body
    }

    #[test]
    fn new_entry_body() {
        let uuid: Uuid = [84_u8; uuid::SIZE].into();
        let username = String::from("Username");
        let password = String::from("Password");

        let body = EntryBody::new(uuid.clone(), username.clone(), password.clone());

        assert_eq!(body.uuid, uuid);
        assert_eq!(body.username, SecString::from(username));
        assert_eq!(body.password, SecString::from(password));
        assert_eq!(body.email, SecString::new());
        assert!(body.modified);
    }

    #[test]
    fn encrypt_and_decrypt_body() {
        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let body = DEFAULT_BODY.to_owned();

        let encrypted: crate::dto::entry::EntryBody = body
            .encrypt(&master_key)
            .expect("Encrypting entry body should not fail.");

        let decrypted = EntryBody::decrypt(&encrypted, &master_key)
            .expect("Decrypting entry body should not fail.");

        assert!(equal_bodies(&body, &decrypted));
    }

    #[test]
    fn save_and_load_body() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        let body = DEFAULT_BODY.to_owned();

        let encrypted = body.encrypt(&master_key).unwrap();
        crate::io::save_entry_body(&path, body.uuid(), &encrypted).unwrap();

        let loaded = EntryBody::load(&path, body.uuid(), &master_key)
            .expect("Loading entry body should not fail.");

        assert!(equal_bodies(&body, &loaded));
    }

    #[test]
    fn set_username() {
        let username = "Custom username";

        let mut body = DEFAULT_BODY.to_owned();
        body.modified = false;

        assert_eq!(body.username, SecString::from("Default username"));

        let _ = body.set_username(username.to_owned());

        assert!(body.modified);
        assert_eq!(body.username, SecString::from(username));
    }

    #[test]
    fn set_password() {
        let password = "Custom password";

        let mut body = DEFAULT_BODY.to_owned();
        body.modified = false;

        assert_eq!(body.password, SecString::from("Default password"));

        let _ = body.set_password(password.to_owned());

        assert!(body.modified);
        assert_eq!(body.password, SecString::from(password));
    }

    #[test]
    fn set_email() {
        let email = "custom@example.web";

        let mut body = DEFAULT_BODY.to_owned();
        body.modified = false;

        assert_eq!(body.email, SecString::from(""));

        let _ = body.set_email(email.to_owned());

        assert!(body.modified);
        assert_eq!(body.email, SecString::from(email));
    }

    #[test]
    fn is_modified_body() {
        let mut body = DEFAULT_BODY.to_owned();

        assert!(body.is_modified());
        body.modified = false;
        assert!(!body.is_modified());
    }

    fn equal_bodies(a: &EntryBody, b: &EntryBody) -> bool {
        a.uuid == b.uuid
            && a.username == b.username
            && a.password == b.password
            && a.email == b.email
    }

    #[test]
    fn encrypted_body_from_and_into() {
        let body = DEFAULT_BODY.to_owned();

        let master_key = [21_u8; cryptography::MASTER_KEY_SIZE];

        cryptography::generate_iv.mock_safe(|len| MockResult::Return(vec![42_u8; len]));
        unsafe {
            cryptography::aes_cbc_encrypt.mock_raw(|data, key, iv| {
                assert_eq!(key, master_key);
                assert_eq!(iv, vec![42_u8; cryptography::AES_IV_LENGTH]);
                MockResult::Continue((data, key, iv))
            });
        }

        let encrypted = EncryptedBody::from(&body, &master_key)
            .expect("Turning EntryBody into EncryptedBody should not fail.");

        let decrypted = encrypted
            .into(&master_key)
            .expect("Turning EncryptedBody into EntryBody should not fail.");

        assert!(equal_bodies(&body, &decrypted));
    }

    #[test]
    fn auto_type_sequence() {
        let default_sequence = AutoTypeSequence::default();

        assert_eq!(default_sequence.as_str(), DEFAULT_SEQUENCE);

        let sequence = AutoTypeSequence::from(String::from("Test Sequence"));
        assert_eq!(sequence.as_str(), "Test Sequence");
    }
}
