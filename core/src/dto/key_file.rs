//! TODO

use getset::Getters;
use serde::{Deserialize, Serialize};

/// The encrypted key file as a data-transfer-object (dto).
#[derive(Debug, Deserialize, Serialize, Getters)]
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
