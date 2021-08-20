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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::io::{create_new_vault_dir, BODY, ENTRIES_DIR, GROUPS_DIR, HEAD};

    #[test]
    fn vault_dir_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let expected = path.join("DuckDuckVault");
        let expected_groups = expected.join(GROUPS_DIR);
        let expected_entries = expected.join(ENTRIES_DIR);
        let expected_entrie_heads = expected_entries.join(HEAD);
        let expected_entrie_bodies = expected_entries.join(BODY);

        assert!(!expected.exists());
        assert!(!expected_groups.exists());
        assert!(!expected_entries.exists());
        assert!(!expected_entrie_heads.exists());
        assert!(!expected_entrie_bodies.exists());

        create_new_vault_dir(&expected).expect("Creation of new vault dir should not fail.");

        assert!(expected.exists());
        assert!(expected_groups.exists());
        assert!(expected_entries.exists());
        assert!(expected_entrie_heads.exists());
        assert!(expected_entrie_bodies.exists());
    }
}
