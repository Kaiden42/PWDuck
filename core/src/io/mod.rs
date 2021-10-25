//! This module contains everything related to IO.
use std::path::Path;

use crate::{
    cryptography::fill_random_bytes,
    model::uuid::{self, Uuid},
};

/// The directory name of the groups
pub const GROUPS_DIR: &str = "groups";

/// The directory name of the entries
pub const ENTRIES_DIR: &str = "entries";

/// The directory name of the entry heads.
pub const HEAD: &str = "head";

/// The directory name of the entry bodies.
pub const BODY: &str = "body";

/// The file name of the master key
pub const MASTER_KEY_NAME: &str = "master_key.pwduck";

/// The directory name of the application settings.
pub const APPLICATION_SETTINGS_DIR: &str = "PWDuck";
/// The file name of the application settings.
pub const APPLICATION_SETTINGS_NAME: &str = "settings.ron";

mod entry;
pub use entry::*;

mod group;
pub use group::*;

mod key_file;
pub use key_file::*;

mod master_key;
pub use master_key::*;

mod settings;
pub use settings::*;

mod vault;
pub use vault::*;

/// Generate a random UUID for the given path.
pub fn generate_uuid(path: &Path) -> Uuid {
    let mut uuid = [0_u8; uuid::SIZE];

    loop {
        fill_random_bytes(&mut uuid);
        let file_name = base64::encode(sha256::digest_bytes(&uuid));

        if !path.join(GROUPS_DIR).join(&file_name).exists()
            && !path.join(ENTRIES_DIR).join(&file_name).exists()
        {
            break;
        }
    }

    uuid.into()
}
