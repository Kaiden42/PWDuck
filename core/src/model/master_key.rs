//! TODO

use getset::Getters;
use std::path::Path;
use zeroize::Zeroize;

use crate::{cryptography::decrypt_masterkey, error::PWDuckCoreError};
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
        let dto = crate::io::load_masterkey(&path)?;
        decrypt_masterkey(&dto, password, key_protection, nonce)
    }
}

impl From<Vec<u8>> for MasterKey {
    fn from(key: Vec<u8>) -> Self {
        Self { key }
    }
}
