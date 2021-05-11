//! TODO
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MasterKey {
    salt: String,
    iv: String,
    encrypted_key: String,
}

impl MasterKey {
    /// TODO
    #[must_use]
    pub const fn new(salt: String, iv: String, encrypted_key: String) -> Self {
        Self {
            salt,
            iv,
            encrypted_key,
        }
    }

    /// TODO
    #[must_use]
    pub fn get_salt(&self) -> &str {
        &self.salt
    }

    /// TODO
    #[must_use]
    pub fn get_iv(&self) -> &str {
        &self.iv
    }

    /// TODO
    #[must_use]
    pub fn get_encrypted_key(&self) -> &str {
        &self.encrypted_key
    }
}
