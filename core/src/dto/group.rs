//! The data-transfer-objects of the groups.
//!
//! It represents the encrypted [`Group`](crate::model::group::Group) that
//! is stored on disk.
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

#[cfg(test)]
mod tests {
    use super::Group;

    #[test]
    fn new_group() {
        let group = Group::new("IV".into(), "CONTENT".into());
        assert_eq!(group.iv(), "IV");
        assert_eq!(group.content(), "CONTENT");
    }
}
