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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{dto::master_key::MasterKey, io::create_new_vault_dir};

    use super::{load_masterkey, save_masterkey};

    #[test]
    fn save_and_load_masterkey() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let master_key = MasterKey::new("SALT".into(), "IV".into(), "ENCRYPTED_KEY".into());

        save_masterkey(&path, master_key.clone()).expect("Saving master key should not fail.");

        let loaded = load_masterkey(&path).expect("Loading master key should not fail.");

        assert_eq!(master_key.salt(), loaded.salt());
        assert_eq!(master_key.iv(), loaded.iv());
        assert_eq!(master_key.encrypted_key(), loaded.encrypted_key());
    }
}
