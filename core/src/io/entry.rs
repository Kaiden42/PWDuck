//! The functions to save, load and delete entries.
use std::{fs, path::Path};

use crate::{
    dto::entry::{EntryBody, EntryHead},
    PWDuckCoreError, Uuid,
};

use super::{BODY, ENTRIES_DIR, HEAD};

/// Save the [`EntryHead`](EntryHead) to disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [Vault](Vault)
///  - The UUID as the identifier of the [`EntryHead`](EntryHead)
///  - The [`EntryHead`](EntryHead) to save
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
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the [`EntryHead`](EntryHead)
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
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the [`EntryBody`](EntryBody)
///  - The [`EntryBody`](EntryBody) to save
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
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the [`EntryBody`](EntryBody)
pub fn load_entry_body(path: &Path, uuid: &Uuid) -> Result<EntryBody, PWDuckCoreError> {
    let file_name = uuid.base64hash();
    let content = fs::read_to_string(path.join(ENTRIES_DIR).join(BODY).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Save the entry to disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the entry
///  - The content of the entry
fn save_entry(path: &Path, uuid: &Uuid, content: String) -> Result<(), PWDuckCoreError> {
    let file_name = uuid.base64hash();
    fs::write(path.join(file_name), content)?;
    Ok(())
}

/// Delete the entry from disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the entry head
///  - The UUID as the identifier of the entry body
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

#[cfg(test)]
mod tests {

    use tempfile::tempdir;

    use crate::{
        dto::entry::{EntryBody, EntryHead},
        io::{create_new_vault_dir, BODY, ENTRIES_DIR, HEAD},
        model::uuid,
        Uuid,
    };

    use super::{
        load_all_entry_heads, load_entry_body, load_entry_head, save_entry_body, save_entry_head,
    };

    #[test]
    fn save_and_load_entry_head() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let head = EntryHead::new("IV".into(), "HEAD".into());
        let uuid: Uuid = [21_u8; uuid::SIZE].into();

        save_entry_head(&path, &uuid, &head).expect("Saving entry head should not fail.");

        let loaded = load_entry_head(&path, &uuid).expect("Loading entry head should not fail.");

        assert_eq!(head.iv(), loaded.iv());
        assert_eq!(head.content(), loaded.content());
    }

    #[test]
    fn save_and_load_all_entry_heads() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let mut heads: Vec<(EntryHead, Uuid)> = (0..=10)
            .into_iter()
            .map(|n| {
                let head = EntryHead::new(format!("IV: {}", n), format!("Head: {}", n));
                let uuid: Uuid = [n; uuid::SIZE].into();

                save_entry_head(&path, &uuid, &head).unwrap();

                (head, uuid)
            })
            .collect();

        let mut loaded =
            load_all_entry_heads(&path).expect("Loading all entry heads should not fail");

        heads.sort_by(|a, b| a.0.iv().cmp(&b.0.iv()));
        loaded.sort_by(|a, b| a.iv().cmp(&b.iv()));

        assert_eq!(heads.len(), loaded.len());
        heads.iter().zip(loaded.iter()).for_each(|((a, _), b)| {
            assert_eq!(a.iv(), b.iv());
            assert_eq!(a.content(), b.content());
        })
    }

    #[test]
    fn save_and_load_entry_body() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let body = EntryBody::new("IV".into(), "BODY".into());
        let uuid: Uuid = [21_u8; uuid::SIZE].into();

        save_entry_body(&path, &uuid, &body).expect("Saving entry body should not fail.");

        let loaded = load_entry_body(&path, &uuid).expect("Loading entry body should not fail.");

        assert_eq!(body.iv(), loaded.iv());
        assert_eq!(body.content(), loaded.content());
    }

    #[test]
    fn delete_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let head = EntryHead::new("IV".into(), "HEAD".into());
        let body = EntryBody::new("IV".into(), "BODY".into());

        let head_uuid: Uuid = [42_u8; uuid::SIZE].into();
        let body_uuid: Uuid = [21_u8; uuid::SIZE].into();

        save_entry_head(&path, &head_uuid, &head).unwrap();
        save_entry_body(&path, &body_uuid, &body).unwrap();

        let head_path = path
            .join(ENTRIES_DIR)
            .join(HEAD)
            .join(head_uuid.base64hash());
        let body_path = path
            .join(ENTRIES_DIR)
            .join(BODY)
            .join(body_uuid.base64hash());

        assert!(head_path.exists());
        assert!(body_path.exists());

        super::delete_entry(&path, &head_uuid, &body_uuid)
            .expect("Deletion of entry should not fail");

        assert!(!head_path.exists());
        assert!(!body_path.exists());
    }
}
