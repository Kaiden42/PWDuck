//! TODO

use std::path::Path;
use zeroize::Zeroize;

use crate::{cryptography::decrypt_masterkey, error::PWDuckCoreError};
/// In-memory encrypted master key
#[derive(Debug, Zeroize)]
#[zeroize(drop)]
pub struct MasterKey {
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
        let dto = crate::io::load_masterkey(&path)?;
        decrypt_masterkey(&dto, password, key_protection, nonce)
    }

    /// TODO
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }
}

impl From<Vec<u8>> for MasterKey {
    fn from(key: Vec<u8>) -> Self {
        Self { key }
    }
}
