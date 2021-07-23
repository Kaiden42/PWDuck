//! TODO

use std::{fs, path::Path};

use crate::{dto::master_key::MasterKey, PWDuckCoreError};

use super::MASTERKEY_NAME;

/// Save the [`MasterKey`](MasterKey) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The [`MasterKey`](MasterKey) to save
pub fn save_masterkey(path: &Path, masterkey: MasterKey) -> Result<(), PWDuckCoreError> {
    fs::write(path.join(MASTERKEY_NAME), ron::to_string(&masterkey)?)?;
    drop(masterkey);
    Ok(())
}

/// Load the [`MasterKey`](MasterKey) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the _`Vault`](Vault)
pub fn load_masterkey(path: &Path) -> Result<MasterKey, PWDuckCoreError> {
    let content = fs::read_to_string(path.join(MASTERKEY_NAME))?;
    Ok(ron::from_str(&content)?)
}
