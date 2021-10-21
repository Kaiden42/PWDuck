//! The data-transfer-objects of the key files.
//!
//! It represents the encrypted [`KeyFile`](crate::model::key_file::KeyFile) that
//! is stored on disk.
use getset::Getters;
use serde::{Deserialize, Serialize};

/// The encrypted key file as a data-transfer-object (dto).
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct KeyFile {
    /// The salt used to derive the encryption key from the user's password.
    #[getset(get = "pub")]
    salt: String,

    /// The iv used for the encryption.
    #[getset(get = "pub")]
    iv: String,

    /// The encrypted content of the key file.
    #[getset(get = "pub")]
    encrypted_key: String,
}

impl KeyFile {
    /// Create a new [`KeyFile`](KeyFile).
    pub const fn new(salt: String, iv: String, encrypted_key: String) -> Self {
        Self {
            salt,
            iv,
            encrypted_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeyFile;

    #[test]
    fn new_key_file() {
        let key_file = KeyFile::new("SALT".into(), "IV".into(), "ENCRYPTED_KEY".into());
        assert_eq!(key_file.salt(), "SALT");
        assert_eq!(key_file.iv(), "IV");
        assert_eq!(key_file.encrypted_key(), "ENCRYPTED_KEY");
    }
}
