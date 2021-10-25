//! The functions to save and load master keys.
use std::{fs, path::Path};

use crate::{dto::master_key::MasterKey, PWDuckCoreError};

use super::MASTER_KEY_NAME;

/// Save the [`MasterKey`](MasterKey) to disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The [`MasterKey`](MasterKey) to save
pub fn save_master_key(path: &Path, master_key: MasterKey) -> Result<(), PWDuckCoreError> {
    fs::write(path.join(MASTER_KEY_NAME), ron::to_string(&master_key)?)?;
    drop(master_key);
    Ok(())
}

/// Load the [`MasterKey`](MasterKey) from disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
pub fn load_master_key(path: &Path) -> Result<MasterKey, PWDuckCoreError> {
    let content = fs::read_to_string(path.join(MASTER_KEY_NAME))?;
    Ok(ron::from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{dto::master_key::MasterKey, io::create_new_vault_dir};

    use super::{load_master_key, save_master_key};

    #[test]
    fn save_and_load_master_key() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = MasterKey::new("SALT".into(), "IV".into(), "ENCRYPTED_KEY".into());

        save_master_key(&path, master_key.clone()).expect("Saving master key should not fail.");

        let loaded = load_master_key(&path).expect("Loading master key should not fail.");

        assert_eq!(master_key.salt(), loaded.salt());
        assert_eq!(master_key.iv(), loaded.iv());
        assert_eq!(master_key.encrypted_key(), loaded.encrypted_key());
    }
}
