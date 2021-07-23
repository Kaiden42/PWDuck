//! TODO

use std::{convert::TryFrom, ops::Deref, path::Path};

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{io::generate_uuid, PWDuckCoreError, SecVec};

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

impl Deref for Uuid {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl From<[u8; SIZE]> for Uuid {
    fn from(id: [u8; SIZE]) -> Self {
        Self { id }
    }
}

impl TryFrom<SecVec<u8>> for Uuid {
    type Error = PWDuckCoreError;

    fn try_from(value: SecVec<u8>) -> Result<Self, Self::Error> {
        if value.len() != SIZE {
            return Err(PWDuckCoreError::Error(
                "TryFrom SecVec to Uuid failed".into(),
            ));
        }

        let mut id = [0_u8; SIZE];
        id.copy_from_slice(&value);
        Ok(id.into())
    }
}
