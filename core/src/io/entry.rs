//! TODO

use std::{fs, path::Path};

use crate::{
    dto::entry::{EntryBody, EntryHead},
    PWDuckCoreError, Uuid,
};

use super::{BODY, ENTRIES_DIR, HEAD};

/// Save the [`EntryHead`](EntryHead) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [Vault](Vault)
///     - The UUID as the identifier of the [`EntryHead`](EntryHead)
///     - The [`EntryHead`](EntryHead) to save
pub fn save_entry_head(
    path: &Path,
    uuid: &Uuid,
    entry_head: &EntryHead,
) -> Result<(), PWDuckCoreError> {
    save_entry(
        &path.join(ENTRIES_DIR).join(HEAD),
        uuid,
        ron::to_string(&entry_head)?,
    )
}

/// Load the [`EntryHead`](EntryHead) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`EntryHead`](EntryHead)
pub fn load_entry_head(path: &Path, uuid: &Uuid) -> Result<EntryHead, PWDuckCoreError> {
    let file_name = uuid.base64hash();
    let content = fs::read_to_string(path.join(ENTRIES_DIR).join(HEAD).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Load all [`EntryHead`](EntryHead)s of a vault.
///
/// It expects:
///     The [`Path`](Path) as the location of the [`Vault`](Vault)
pub fn load_all_entry_heads(path: &Path) -> Result<Vec<EntryHead>, PWDuckCoreError> {
    let directory = path.join(ENTRIES_DIR).join(HEAD);

    // TODO: Better error handling
    fs::read_dir(directory)?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|file| fs::read_to_string(file.path()))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|content| ron::from_str(content).map_err(PWDuckCoreError::from))
        .collect()
}

/// Save the [`EntryBody`](EntryBody) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`EntryBody`](EntryBody)
///     - The [`EntryBody`](EntryBody) to save
pub fn save_entry_body(
    path: &Path,
    uuid: &Uuid,
    entry_body: &EntryBody,
) -> Result<(), PWDuckCoreError> {
    save_entry(
        &path.join(ENTRIES_DIR).join(BODY),
        uuid,
        ron::to_string(entry_body)?,
    )
}

/// Load the [`EntryBody`](EntryBody) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`EntryBody`](EntryBody)
pub fn load_entry_body(path: &Path, uuid: &Uuid) -> Result<EntryBody, PWDuckCoreError> {
    let file_name = uuid.base64hash();
    let content = fs::read_to_string(path.join(ENTRIES_DIR).join(BODY).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Save the entry to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the entry
///     - The content of the entry
fn save_entry(path: &Path, uuid: &Uuid, content: String) -> Result<(), PWDuckCoreError> {
    let file_name = uuid.base64hash();
    fs::write(path.join(file_name), content)?;
    Ok(())
}

/// TODO
pub fn delete_entry(
    path: &Path,
    head_uuid: &Uuid,
    body_uuid: &Uuid,
) -> Result<(), PWDuckCoreError> {
    let head_path = path
        .join(ENTRIES_DIR)
        .join(HEAD)
        .join(head_uuid.base64hash());

    if head_path.exists() {
        fs::remove_file(head_path)?;
    }

    let body_path = path
        .join(ENTRIES_DIR)
        .join(BODY)
        .join(body_uuid.base64hash());

    if body_path.exists() {
        fs::remove_file(body_path)?;
    }
    Ok(())
}
