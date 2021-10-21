//! The functions to save and load key files.
use std::{fs, path::Path};

use crate::{dto::key_file::KeyFile, PWDuckCoreError};

/// Save the [`KeyFile`](KeyFile) to disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`KeyFile`](KeyFile)
///  - The [`KeyFile`](KeyFile) to save
pub fn save_key_file(path: &Path, key_file: KeyFile) -> Result<(), PWDuckCoreError> {
    fs::write(path, ron::to_string(&key_file)?)?;
    drop(key_file);
    Ok(())
}

/// Load the [`KeyFile`](KeyFile) from disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`KeyFile`](KeyFile)
pub fn load_key_file(path: &Path) -> Result<KeyFile, PWDuckCoreError> {
    let content = fs::read_to_string(path)?;
    Ok(ron::from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::dto::key_file::KeyFile;

    use super::{load_key_file, save_key_file};

    #[test]
    fn save_and_load_key_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("KeyFile.pwdk");

        let key_file = KeyFile::new("SALT".into(), "IV".into(), "ENCRYPTED_KEY".into());

        save_key_file(&path, key_file.clone()).expect("Saving key file should not fail.");

        let loaded = load_key_file(&path).expect("Loading key file should not fail.");

        assert_eq!(key_file.salt(), loaded.salt());
        assert_eq!(key_file.iv(), loaded.iv());
        assert_eq!(key_file.encrypted_key(), loaded.encrypted_key());
    }
}
