//! TODO

use std::fs;

use crate::{ApplicationSettings, PWDuckCoreError};

use super::{APPLICATION_SETTINGS_DIR, APPLICATION_SETTINGS_NAME};

/// Save the [`ApplicationSettings`](ApplicationSettings) to disk.
pub fn save_application_settings(
    application_settings: &ApplicationSettings,
) -> Result<(), PWDuckCoreError> {
    let path = dirs::config_dir().ok_or_else(|| {
        PWDuckCoreError::Error("Could not find the config directory of the user.".into())
    })?;
    let path = path
        .join(APPLICATION_SETTINGS_DIR)
        .join(APPLICATION_SETTINGS_NAME);

    let content = ron::to_string(application_settings)?;
    fs::write(path, content)?;
    Ok(())
}

/// Load the [`ApplicationSettings`](ApplicationSettings) from disk.
pub fn load_application_settings() -> Result<ApplicationSettings, PWDuckCoreError> {
    let path = dirs::config_dir().ok_or_else(|| {
        PWDuckCoreError::Error("Could not find the config directory of the user.".into())
    })?;
    let path = path.join(APPLICATION_SETTINGS_DIR);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    let path = path.join(APPLICATION_SETTINGS_NAME);
    let content = fs::read_to_string(path)?;
    Ok(ron::from_str(&content)?)
}
