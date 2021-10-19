//! TODO

use std::{fs, path::Path};

use crate::{dto::key_file::KeyFile, PWDuckCoreError};

/// Save the [`KeyFile`](KeyFile) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`KeyFile`](KeyFile)
///     - The [`KeyFile`](KeyFile) to save
pub fn save_key_file(path: &Path, key_file: KeyFile) -> Result<(), PWDuckCoreError> {
    fs::write(path, ron::to_string(&key_file)?)?;
    drop(key_file);
    Ok(())
}

/// Load the [`KeyFile`](KeyFile) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`KeyFile`](KeyFile)
pub fn load_key_file(path: &Path) -> Result<KeyFile, PWDuckCoreError> {
    let content = fs::read_to_string(path)?;
    Ok(ron::from_str(&content)?)
}
