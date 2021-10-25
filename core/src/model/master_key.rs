//! In-memory encrypted master keys stored in memory.
use getset::Getters;
use std::path::Path;
use zeroize::Zeroize;

use crate::{
    cryptography::{decrypt_master_key, derive_key_protection, unprotect_master_key},
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
    ///  - The [`Path`](Path) as the location of the [`Vault`](crate::Vault).
    ///  - The password to decrypt the [`MasterKey`](MasterKey).
    ///  - The key protection to protect the [`MasterKey`](MasterKey) in memory.
    ///  - The nonce used to encrypt the [`MasterKey`](MasterKey) in memory.
    pub fn load(
        path: &Path,
        password: &str,
        key_file: Option<&Path>,
        key_protection: &[u8],
        nonce: &[u8],
    ) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_master_key(path)?;
        decrypt_master_key(&dto, password, key_file, key_protection, nonce)
    }

    /// Decrypt the in-memory encrypted masterkey to receive the unprotected key data.
    ///
    /// It expects:
    ///  - The [`MemKey`](MemKey) used for the in-memory encryption
    ///  - The salt to derive the key for the encryption from the [`MemKey`](MemKey)
    ///  - The nonce used to decrypt the masterkey
    pub fn as_unprotected(
        &self,
        mem_key: &MemKey,
        salt: &[u8],
        nonce: &[u8],
    ) -> Result<SecVec<u8>, PWDuckCoreError> {
        let key_protection = derive_key_protection(mem_key, salt)?;
        let master_key = unprotect_master_key(&self.key, &key_protection, nonce)?;
        Ok(master_key)
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

#[cfg(test)]
mod tests {
    use mocktopus::mocking::*;
    use seckey::SecBytes;
    use tempfile::tempdir;

    use crate::{
        cryptography::{self, generate_master_key},
        io::create_new_vault_dir,
        MemKey,
    };

    use super::MasterKey;

    #[test]
    fn load_and_unprotect_master_key() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        cryptography::fill_random_bytes.mock_safe(|buf| {
            buf.fill(0_u8);
            MockResult::Return(())
        });

        let password = "This is a totally secret password";
        let master_key = generate_master_key(&password, None).unwrap();
        crate::io::save_master_key(&path, master_key.clone()).unwrap();

        MemKey::with_length.mock_safe(|len| {
            MockResult::Return(SecBytes::with(len, |buf| buf.fill(255_u8)).into())
        });
        let mem_key = MemKey::new();
        let nonce = [42_u8; cryptography::CHACHA20_NONCE_LENGTH];
        let salt = cryptography::generate_salt();

        let key_protection = cryptography::derive_key_protection(&mem_key, &salt).unwrap();

        let loaded = MasterKey::load(&path, &password, None, &key_protection, &nonce)
            .expect("Loading master key should not fail.");

        let unprotected = loaded
            .as_unprotected(&mem_key, &salt, &nonce)
            .expect("Unprotect master key should not fail.");

        assert_eq!(
            unprotected.as_slice(),
            &[0_u8; cryptography::MASTER_KEY_SIZE]
        );
    }
}
