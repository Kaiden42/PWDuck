//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};
/// The encrypted masterkey as a data-transfer-object (dto).
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct MasterKey {
    /// The salt used to derive the encryption key from the user's password.
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
