//! TODO

use std::{fs, path::Path};

use crate::PWDuckCoreError;

use super::{BODY, ENTRIES_DIR, GROUPS_DIR, HEAD};

/// Create the directory structure of a new [Vault](Vault) on the given path.
pub fn create_new_vault_dir(path: &Path) -> Result<(), PWDuckCoreError> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join(GROUPS_DIR))?;
    fs::create_dir_all(path.join(ENTRIES_DIR))?;
    fs::create_dir_all(path.join(ENTRIES_DIR).join(HEAD))?;
    fs::create_dir_all(path.join(ENTRIES_DIR).join(BODY))?;
    Ok(())
}
