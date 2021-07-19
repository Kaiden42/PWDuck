//! TODO

use std::path::Path;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::io::generate_uuid;

/// The size of an [`Uuid`](Uuid).
pub const SIZE: usize = 16;

/// Universally Unique Identifier (UUID) of each data element of the [`Vault`](crate::Vault).
///
/// See: [`Group`](crate::Group), [`EntryHead`](crate::EntryHead) and [`EntryBody`](crate::EntryBody).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Zeroize)]
pub struct Uuid {
    /// The UUID.
    id: [u8; SIZE],
}

impl Uuid {
    /// Generate a new UUID for the given path.
    #[must_use]
    pub fn new(path: &Path) -> Self {
        generate_uuid(path)
    }

    /// Returns the Base64 encoded SHA256 hash of this [`Uuid`](Uuid).
    #[must_use]
    pub fn base64hash(&self) -> String {
        base64::encode(sha256::digest_bytes(&self.id))
    }
}

impl From<[u8; SIZE]> for Uuid {
    fn from(id: [u8; SIZE]) -> Self {
        Self { id }
    }
}
