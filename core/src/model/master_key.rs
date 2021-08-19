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
    /// The key data.
    #[getset(get = "pub")]
    key: Vec<u8>,
}

impl MasterKey {
    /// Load a [`MasterKey`](MasterKey) from disk.
    ///
    /// It expects:
    ///     - The [`Path`](Path) as the location of the [`Vault`](crate::Vault).
    ///     - The password to decrypt the [`MasterKey`](MasterKey).
    pub fn load(
        path: &Path,
        password: &str,
        key_protection: &[u8],
        nonce: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_masterkey(path)?;
        decrypt_masterkey(&dto, password, key_protection, nonce)
    }

    /// Decrypt the in-memory encrypted masterkey to receive the unprotected key data.
    ///
    /// It expects:
    ///     - The [`MemKey`](MemKey) used for the in-memory encryption
    ///     - The salt to derive the key for the encryption from the [`MemKey`](MemKey)
    ///     - The nonce used to decrypt the masterkey
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

impl std::ops::Deref for MasterKey {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl From<Vec<u8>> for MasterKey {
    fn from(key: Vec<u8>) -> Self {
        Self { key }
    }
}
