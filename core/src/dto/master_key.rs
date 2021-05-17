//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};
/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct MasterKey {
    /// TODO
    #[getset(get = "pub")]
    salt: String,

    /// TODO
    #[getset(get = "pub")]
    iv: String,

    /// TODO
    #[getset(get = "pub")]
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
}
