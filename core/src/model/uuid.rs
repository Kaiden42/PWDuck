//! The unique identifiers of each data in a vault.
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
    #[must_use] // TODO: Snakecase!
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

#[cfg(test)]
mod tests {

    use std::convert::TryFrom;

    use mocktopus::mocking::*;
    use tempfile::tempdir;

    use crate::{cryptography, io::create_new_vault_dir, SecVec};

    use super::{Uuid, SIZE};

    #[test]
    fn new_uuid() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        cryptography::fill_random_bytes.mock_safe(|buf| {
            buf.fill(42_u8);
            MockResult::Return(())
        });

        let uuid = Uuid::new(&path);

        assert_eq!(uuid.as_ref(), &[42_u8; SIZE]);
    }

    #[test]
    fn base64hash() {
        let uuid: Uuid = [21_u8; SIZE].into();

        let hash = "ZDkyMTBjZmUyNzljYzIxZGM2ODdlNmJkODAyMmZlOWY1YWU0NjA3Y2MyZDg3OWNmMGMwNGY5OGRiMmFkOGJhYw==";

        assert_eq!(hash, uuid.base64hash().as_str());
    }

    #[test]
    fn try_from() {
        let valid: SecVec<u8> = vec![21_u8; SIZE].into();

        let _ = Uuid::try_from(valid).expect("Creating uuid from valid vec should not fail.");

        let invalid: SecVec<u8> = vec![21_u8; SIZE + 1].into();

        let _ = Uuid::try_from(invalid).expect_err("Creating uuid from invalid vec should fail.");

        let invalid: SecVec<u8> = vec![21_u8; SIZE - 1].into();

        let _ = Uuid::try_from(invalid).expect_err("Creating uuid from invalid vec should fail.");
    }
}
