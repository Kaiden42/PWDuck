//! TODO

use std::{fs, path::PathBuf};

use crate::{ApplicationSettings, PWDuckCoreError};

use super::{APPLICATION_SETTINGS_DIR, APPLICATION_SETTINGS_NAME};

#[cfg(test)]
use mocktopus::macros::*;

/// Save the [`ApplicationSettings`](ApplicationSettings) to disk.
pub fn save_application_settings(
    application_settings: &ApplicationSettings,
) -> Result<(), PWDuckCoreError> {
    //let path = dirs::config_dir().ok_or_else(|| {
    //    PWDuckCoreError::Error("Could not find the config directory of the user.".into())
    //})?;

    //let path = get_system_config_dir()?;
    //let path = path
    //    .join(APPLICATION_SETTINGS_DIR);
    //if !path.exists() {
    //    fs::create_dir_all(&path)?;
    //}
    //let path = path.join(APPLICATION_SETTINGS_NAME);
    let path = get_settings_dir()?.join(APPLICATION_SETTINGS_NAME);

    let content = ron::to_string(application_settings)?;
    fs::write(path, content)?;
    Ok(())
}

/// Load the [`ApplicationSettings`](ApplicationSettings) from disk.
pub fn load_application_settings() -> Result<ApplicationSettings, PWDuckCoreError> {
    //let path = dirs::config_dir().ok_or_else(|| {
    //    PWDuckCoreError::Error("Could not find the config directory of the user.".into())
    //})?;

    //let path = get_system_config_dir()?;
    //let path = path.join(APPLICATION_SETTINGS_DIR);
    //if !path.exists() {
    //    fs::create_dir_all(&path)?;
    //}
    //let path = path.join(APPLICATION_SETTINGS_NAME);
    let path = get_settings_dir()?.join(APPLICATION_SETTINGS_NAME);

    let content = fs::read_to_string(path)?;
    Ok(ron::from_str(&content)?)
}

/// get the path to the settings directory. The directory will be created if it not exists.
fn get_settings_dir() -> Result<PathBuf, PWDuckCoreError> {
    let path = get_system_config_dir()?;
    let path = path.join(APPLICATION_SETTINGS_DIR);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

/// Get the config directory that is specified by the system.
#[cfg_attr(test, mockable)]
#[cfg_attr(coverage, no_coverage)]
fn get_system_config_dir() -> Result<PathBuf, PWDuckCoreError> {
    dirs::config_dir().ok_or_else(|| {
        PWDuckCoreError::Error("Could not find the config directory of the user.".into())
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use mocktopus::mocking::*;

    use crate::{
        io::{settings::get_settings_dir, APPLICATION_SETTINGS_DIR},
        load_application_settings, save_application_settings, ApplicationSettings,
    };

    use super::get_system_config_dir;

    #[test]
    fn settings_dir_creation() {
        let dir = tempdir().unwrap();
        unsafe {
            get_system_config_dir.mock_raw(|| MockResult::Return(Ok(dir.path().to_path_buf())));
        }

        let expected_path = dir.path().join(APPLICATION_SETTINGS_DIR);
        assert!(!expected_path.exists());

        let created = get_settings_dir().expect("Creation of settings dir should not fail.");

        assert!(expected_path.exists());
        assert_eq!(created, expected_path);
    }

    #[test]
    fn save_and_load_application_settings() {
        //let dir = tempdir().unwrap();
        //let path = dir.path();
        //create_new_vault_dir(&path).unwrap();
        let dir = tempdir().unwrap();
        unsafe {
            get_system_config_dir.mock_raw(|| MockResult::Return(Ok(dir.path().to_path_buf())));
        }

        let settings = ApplicationSettings::default();

        save_application_settings(&settings).expect("Saving settings should not fail.");

        let loaded = load_application_settings().expect("Loading settings should not fail.");

        assert_eq!(settings.theme(), loaded.theme());
    }
}
