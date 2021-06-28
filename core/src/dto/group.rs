//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};
/// An encrypted [`Group`] of passwords and sub-groups as a data-transfer-object (dto).
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct Group {
    /// The iv used to encrypt this group.
    #[getset(get = "pub")]
    iv: String,

    /// The encrypted content of this group.
    #[getset(get = "pub")]
    content: String,
}

impl Group {
    /// Create a new [`Group`](Group).
    pub const fn new(iv: String, content: String) -> Self {
        Self { iv, content }
    }
}
