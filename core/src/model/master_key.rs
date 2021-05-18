//! TODO

use getset::Getters;
use std::path::Path;
use zeroize::Zeroize;

use crate::{
    cryptography::{decrypt_masterkey, derive_key_protection, unprotect_masterkey},
    error::PWDuckCoreError,
    MemKey, SecVec,
};
/// In-memory encrypted master key
#[derive(Clone, Debug, Zeroize)]
#[zeroize(drop)]
#[derive(Getters)]
pub struct MasterKey {
    /// TODO
    #[getset(get = "pub")]
    key: Vec<u8>,
}

impl MasterKey {
    /// TODO
    pub fn load(
        path: &Path,
        password: &str,
        key_protection: &[u8],
        nonce: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_masterkey(path)?;
        decrypt_masterkey(&dto, password, key_protection, nonce)
    }

    /// TODO
    pub fn as_unprotected(
        &self,
        mem_key: &MemKey,
        salt: &str,
        nonce: &[u8],
    ) -> Result<SecVec<u8>, PWDuckCoreError> {
        let key_protection = derive_key_protection(mem_key, salt)?;
        let masterkey = unprotect_masterkey(&self.key, &key_protection, nonce)?;
        Ok(masterkey)
    }
}

impl From<Vec<u8>> for MasterKey {
    fn from(key: Vec<u8>) -> Self {
        Self { key }
    }
}
