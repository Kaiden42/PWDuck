//! The data-transfer-objects of the master keys.
//!
//! It represents the encrypted [`MasterKey`](crate::model::master_key::MasterKey)
//! that is stored on disk.
use getset::Getters;
use serde::{Deserialize, Serialize};
/// The encrypted masterkey as a data-transfer-object (dto).
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct MasterKey {
    /// The salt used to derive the encryption key from the user's password
    /// or from the keyfile.
    #[getset(get = "pub")]
    salt: String,

    /// The iv used for the encryption.
    #[getset(get = "pub")]
    iv: String,

    /// The encrypted content of the masterkey.
    #[getset(get = "pub")]
    encrypted_key: String,
}

impl MasterKey {
    /// Create a new [`MasterKey`](MasterKey).
    #[must_use]
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
    use super::MasterKey;

    #[test]
    fn new_master_key() {
        let master_key = MasterKey::new("SALT".into(), "IV".into(), "ENCRYPTED_KEY".into());
        assert_eq!(master_key.salt(), "SALT");
        assert_eq!(master_key.iv(), "IV");
        assert_eq!(master_key.encrypted_key(), "ENCRYPTED_KEY");
    }
}
