//! TODO
use std::{collections::HashMap, path::Path};

use getset::Getters;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{
    cryptography::{aes_cbc_decrypt, aes_cbc_encrypt, generate_aes_iv},
    error::PWDuckCoreError,
    mem_protection::SecString,
};

use super::uuid::Uuid;

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
#[derive(Getters)]
pub struct Group {
    /// TODO
    #[getset(get = "pub")]
    uuid: Uuid,

    /// TODO
    #[getset(get = "pub")]
    parent: String,

    /// TODO
    #[getset(get = "pub")]
    title: String,

    #[serde(skip)]
    modified: bool,
}

impl Group {
    /// TODO
    pub fn new(uuid: Uuid, parent: String, title: String) -> Self {
        Self {
            uuid,
            parent,
            title,
            modified: true,
        }
    }

    /// TODO
    pub fn save(&mut self, path: &Path, masterkey: &[u8]) -> Result<(), PWDuckCoreError> {
        let group = self.encrypt(masterkey)?;
        crate::io::save_group(path, &self.uuid.as_string(), group)?;
        self.modified = false;
        Ok(())
    }

    /// TODO
    fn encrypt(&self, masterkey: &[u8]) -> Result<crate::dto::group::Group, PWDuckCoreError> {
        let iv = generate_aes_iv();
        let mut content = ron::to_string(self)?;
        let encrypted_content = aes_cbc_encrypt(&content.as_bytes(), masterkey, &iv)?;
        content.zeroize();
        Ok(crate::dto::group::Group::new(
            base64::encode(iv),
            base64::encode(encrypted_content),
        ))
    }

    /// TODO
    pub fn load(path: &Path, uuid: &str, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_group(&path, uuid)?;
        Self::decrypt(dto, masterkey)
    }

    /// TODO
    pub fn load_all(
        path: &Path,
        masterkey: &[u8],
    ) -> Result<HashMap<String, Self>, PWDuckCoreError> {
        let dtos = crate::io::load_all_groups(&path)?;

        //let mut results = Vec::with_capacity(dtos.len());
        let mut results = HashMap::new();

        for dto in dtos {
            //results.push(Self::decrypt(dto, masterkey)?);
            let group = Self::decrypt(dto, masterkey)?;
            let _ = results.insert(group.uuid().as_string(), group);
        }

        Ok(results)
    }

    fn decrypt(dto: crate::dto::group::Group, masterkey: &[u8]) -> Result<Self, PWDuckCoreError> {
        let decrypted_content = aes_cbc_decrypt(
            &base64::decode(dto.content())?,
            masterkey,
            &base64::decode(dto.iv())?,
        )?;

        let content = SecString::from_utf8(decrypted_content)?;
        let group = ron::from_str(&content)?;

        Ok(group)
    }

    /// TODO
    pub fn create_root_for(path: &Path) -> Self {
        Self {
            uuid: Uuid::new(path),
            parent: String::new(),
            title: String::new(),
            modified: true,
        }
    }

    /// TODO
    pub fn is_root(&self) -> bool {
        self.parent.is_empty()
    }

    /// TODO
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
