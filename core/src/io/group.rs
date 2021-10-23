//! The functions to save, load and delete groups.
use std::{fs, path::Path};

use crate::{dto::group::Group, PWDuckCoreError, Uuid};

use super::GROUPS_DIR;

/// Save the [`Group`](Group) to disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the [`Group`](Group)
///  - The [`Group`](Group) to save
pub fn save_group(path: &Path, uuid: &Uuid, group: &Group) -> Result<(), PWDuckCoreError> {
    let file_name = uuid.base64_hash();
    fs::write(
        path.join(GROUPS_DIR).join(file_name),
        ron::to_string(&group)?,
    )?;
    Ok(())
}

/// Delete the [`Group`](Group) from disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the [`Group`](Group)
pub fn delete_group(path: &Path, uuid: &Uuid) -> Result<(), PWDuckCoreError> {
    let group_path = path.join(GROUPS_DIR).join(uuid.base64_hash());
    if group_path.exists() {
        fs::remove_file(group_path)?;
    }
    Ok(())
}

/// Load the [`Group`](Group) from disk.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
///  - The UUID as the identifier of the [`Group`](Group)
pub fn load_group(path: &Path, uuid: &Uuid) -> Result<Group, PWDuckCoreError> {
    let file_name = uuid.base64_hash();
    let content = fs::read_to_string(path.join(GROUPS_DIR).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Load all [`Group`](Group)s of a vault.
///
/// It expects:
///  - The [`Path`](Path) as the location of the [`Vault`](Vault)
pub fn load_all_groups(path: &Path) -> Result<Vec<Group>, PWDuckCoreError> {
    let directory = path.join(GROUPS_DIR);

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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        dto::group::Group,
        io::{create_new_vault_dir, GROUPS_DIR},
        model::uuid,
        Uuid,
    };

    use super::{load_all_groups, load_group, save_group};

    #[test]
    fn save_and_load_group() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let group = Group::new("IV".into(), "CONTENT".into());
        let uuid: Uuid = [21_u8; uuid::SIZE].into();

        save_group(&path, &uuid, &group).expect("Saving group should not fail.");

        let loaded = load_group(&path, &uuid).expect("Loading group should not fail.");

        assert_eq!(group.iv(), loaded.iv());
        assert_eq!(group.content(), loaded.content());
    }

    #[test]
    fn save_and_load_all_groups() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let mut groups: Vec<(Group, Uuid)> = (0..=10)
            .into_iter()
            .map(|n| {
                let group = Group::new(format!("IV: {}", n), format!("Group: {}", n));
                let uuid: Uuid = [n; uuid::SIZE].into();

                save_group(&path, &uuid, &group).unwrap();

                (group, uuid)
            })
            .collect();

        let mut loaded = load_all_groups(&path).expect("Loading all groups should not fail.");

        groups.sort_by(|a, b| a.0.iv().cmp(&b.0.iv()));
        loaded.sort_by(|a, b| a.iv().cmp(&b.iv()));

        assert_eq!(groups.len(), loaded.len());
        groups.iter().zip(loaded.iter()).for_each(|((a, _), b)| {
            assert_eq!(a.iv(), b.iv());
            assert_eq!(a.content(), b.content());
        })
    }

    #[test]
    fn delete_group() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        create_new_vault_dir(&path).unwrap();

        let group = Group::new("IV".into(), "CONTENT".into());
        let uuid: Uuid = [42_u8; uuid::SIZE].into();

        save_group(&path, &uuid, &group).unwrap();

        let group_path = path.join(GROUPS_DIR).join(uuid.base64_hash());

        assert!(group_path.exists());

        super::delete_group(&path, &uuid).expect("Deletion of group should not fail.");

        assert!(!group_path.exists());
    }
}
