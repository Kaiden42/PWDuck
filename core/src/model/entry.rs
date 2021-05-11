//! TODO
use std::{collections::HashMap, path::Path};

use crate::{
    cryptography::{aes_cbc_decrypt, aes_cbc_encrypt, generate_aes_iv},
    error::PWDuckCoreError,
    mem_protection::SecString,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use super::uuid::Uuid;

/// TODO
#[derive(Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct EntryHead {
    uuid: Uuid,
    parent: String,
    title: String,
    body: String,
    #[serde(skip)]
    modified: bool,
}

impl EntryHead {
    /// TODO
    pub fn save(&mut self, path: &Path, masterkey: &[u8]) -> Result<(), PWDuckCoreError> {
        let entry_head = self.encrypt(masterkey)?;
        crate::io::save_entry_head(path, &self.uuid.as_string(), entry_head)?;
        self.modified = false;
        Ok(())
    }

    /// TODO
    fn encrypt(&self, masterkey: &[u8]) -> Result<crate::dto::entry::EntryHead, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(&content.as_bytes(), masterkey, &iv)?;
        content.zeroize();
        Ok(crate::dto::entry::EntryHead::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// TODO
    pub fn load(path: &Path, uuid: &str, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_head(&path, uuid)?;
        Self::decrypt(dto, masterkey)
    }

    /// TODO
    pub fn load_all(
        path: &Path,
        masterkey: &[u8],
    ) -> Result<HashMap<String, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_entry_heads(&path)?;

        //let mut results = Vec::with_capacity(dtos.len());
        let mut results = HashMap::new();

        for dto in dtos {
            //results.push(Self::decrypt(dto, masterkey)?);
            let head = Self::decrypt(dto, masterkey)?;
            let _ = results.insert(head.get_uuid().as_string(), head);
        }

        Ok(results)
    }

    fn decrypt(
        dto: crate::dto::entry::EntryHead,
        masterkey: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.get_content())?,
            masterkey,
            &base64::decode(dto.get_iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;
        let head = ron::from_str(&content)?;

        Ok(head)
    }

    /// TODO
    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// TODO
    pub fn get_parent(&self) -> &str {
        &self.parent
    }

    /// TODO
    pub fn get_title(&self) -> &str {
        &self.title
    }

    /// TODO
    pub fn get_body(&self) -> &str {
        &self.body
    }

    /// TODO
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}

/// TODO
#[allow(missing_debug_implementations)]
#[derive(Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct EntryBody {
    uuid: String,
    username: String,
    password: String,
    #[serde(skip)]
    modified: bool,
}

impl EntryBody {
    /// TODO
    pub fn encrypt(
        &self,
        master_key: &[u8],
    ) -> Result<crate::dto::entry::EntryBody, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(&content.as_bytes(), master_key, &iv)?;
        content.zeroize();
        Ok(crate::dto::entry::EntryBody::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// TODO
    pub fn load(path: &Path, uuid: &str, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_entry_body(&path, uuid)?;
        let body = Self::decrypt(dto, masterkey)?;
        Ok(body)
    }

    /// TODO
    pub fn decrypt(
        dto: crate::dto::entry::EntryBody,
        masterkey: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.get_content())?,
            masterkey,
            &base64::decode(dto.get_iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;
        let body = ron::from_str(&content)?;

        Ok(body)
    }

    /// TODO
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    /// TODO
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// TODO
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// TODO
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
