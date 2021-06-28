//! TODO

use std::path::Path;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::io::generate_uuid;
/// Universally Unique Identifier (UUID) of each data element of the [`Vault`](crate::Vault).
///
/// See: [`Group`](crate::Group), [`EntryHead`](crate::EntryHead) and [`EntryBody`](crate::EntryBody).
#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
pub struct Uuid {
    /// The UUID.
    id: Vec<u8>,
}

impl Uuid {
    /// Generate a new UUID for the given path.
    #[must_use]
    pub fn new(path: &Path) -> Self {
        generate_uuid(path)
    }

    /// Returns the Base64 encoded SHA-256 hash of this [`Uuid`](Uuid).
    #[must_use]
    pub fn as_string(&self) -> String {
        base64::encode(sha256::digest_bytes(&self.id))
    }
}

impl From<Vec<u8>> for Uuid {
    fn from(v: Vec<u8>) -> Self {
        Self { id: v }
    }
}
