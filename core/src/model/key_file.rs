//! TODO

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
    ///     - The [`Path`](Path) as the location of the [`KeyFile`](KeyFile).
    ///     - The password to decrypt the [`KeyFile`](KeyFile).
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

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl From<SecVec<u8>> for KeyFile {
    fn from(key: SecVec<u8>) -> Self {
        Self { key }
    }
}
