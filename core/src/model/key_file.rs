//! Decrypted key files stored in memory.
use std::path::Path;
use zeroize::Zeroize;

use getset::Getters;

use crate::{cryptography::decrypt_key_file, PWDuckCoreError, SecVec};

/// Key file used as a 2nd factor.
#[derive(Getters, Zeroize)]
#[zeroize(drop)]
pub struct KeyFile {
    /// The key data.
    #[getset(get = "pub")]
    key: SecVec<u8>,
}

impl KeyFile {
    /// Load a [`KeyFile`](KeyFile) from disk.
    ///
    /// It expects:
    ///  - The [`Path`](Path) as the location of the [`KeyFile`](KeyFile).
    ///  - The password to decrypt the [`KeyFile`](KeyFile).
    pub fn load(path: &Path, password: &str) -> Result<Self, PWDuckCoreError> {
        let dto = crate::io::load_key_file(path)?;
        decrypt_key_file(&dto, password)
    }
}

impl std::fmt::Debug for KeyFile {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("This is a KeyFile")
    }
}

impl std::ops::Deref for KeyFile {
    type Target = Vec<u8>;

    #[cfg_attr(coverage, no_coverage)]
    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl From<SecVec<u8>> for KeyFile {
    #[cfg_attr(coverage, no_coverage)]
    fn from(key: SecVec<u8>) -> Self {
        Self { key }
    }
}

#[cfg(test)]
mod tests {
    use mocktopus::mocking::*;
    use tempfile::tempdir;

    use crate::cryptography::{self, generate_key_file};

    use super::KeyFile;

    #[test]
    fn load_key_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("KeyFile.pwdk");

        cryptography::fill_random_bytes.mock_safe(|buf| {
            buf.fill(42_u8);
            MockResult::Return(())
        });

        let password = "This is a totally secret password";
        let key_file = generate_key_file(&password, &path).unwrap();

        let loaded = KeyFile::load(&path, &password)
            .expect("Loading and decrypting key file should not fail.");

        assert_eq!(loaded.key().as_slice(), key_file.key().as_slice());
    }
}
