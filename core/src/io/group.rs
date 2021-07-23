//! TODO

use std::{fs, path::Path};

use crate::{dto::group::Group, PWDuckCoreError, Uuid};

use super::GROUPS_DIR;

/// Save the [`Group`](Group) to disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`Group`](Group)
///     - The [`Group`](Group) to save
pub fn save_group(path: &Path, uuid: &Uuid, group: &Group) -> Result<(), PWDuckCoreError> {
    let file_name = uuid.base64hash();
    fs::write(
        path.join(GROUPS_DIR).join(file_name),
        ron::to_string(&group)?,
    )?;
    Ok(())
}

/// TODO
pub fn delete_group(path: &Path, uuid: &Uuid) -> Result<(), PWDuckCoreError> {
    let group_path = path.join(GROUPS_DIR).join(uuid.base64hash());
    if group_path.exists() {
        fs::remove_file(group_path)?;
    }
    Ok(())
}

/// Load the [`Group`](Group) from disk.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
///     - The UUID as the identifier of the [`Group`](Group)
pub fn load_group(path: &Path, uuid: &Uuid) -> Result<Group, PWDuckCoreError> {
    let file_name = uuid.base64hash();
    let content = fs::read_to_string(path.join(GROUPS_DIR).join(file_name))?;
    Ok(ron::from_str(&content)?)
}

/// Load all [`Group`](Group)s of a vault.
///
/// It expects:
///     - The [`Path`](Path) as the location of the [`Vault`](Vault)
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
