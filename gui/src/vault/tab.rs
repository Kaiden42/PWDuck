//! The view and management of a tab.
use std::path::PathBuf;

use iced::Command;
use iced_focus::Focus;
use pwduck_core::Vault;

use crate::{
    error::PWDuckGuiError, theme::Theme, vault::container::ToolBarMessage, Component, Platform,
    Viewport,
};

pub use super::container::VaultContainerMessage;
use super::{
    container::VaultContainer,
    creator::{VaultCreator, VaultCreatorMessage},
    loader::{VaultLoader, VaultLoaderMessage},
    settings::{Settings, SettingsMessage},
    unlock::{VaultUnlocker, VaultUnlockerMessage},
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of a vault tab.
#[derive(Debug, Focus)]
pub struct VaultTab {
    /// The state of the tab content.
    #[focus(enable)]
    state: VaultTabState,
}

#[cfg_attr(test, mockable)]
impl VaultTab {
    /// True, if the tab contains unsaved changes.
    #[must_use]
    pub fn contains_unsaved_changes(&self) -> bool {
        match &self.state {
            VaultTabState::Open(container) => container.contains_unsaved_changes(),
            _ => false,
        }
    }

    /// Change the content of the tab to the [`VaultCreator`](VaultCreator).
    fn change_to_create_state(&mut self) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Create(Box::new(VaultCreator::new(())));
        Command::none()
    }

    /// Change the content of the tab to the [`VaultLoader`](VaultLoader).
    fn change_to_empty_state(&mut self) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Empty(VaultLoader::new(()));
        Command::none()
    }

    /// Change the content of the tab to the [`VaultUnlocker`](VaultUnlocker).
    fn change_to_unlock_state(
        &mut self,
        path: PathBuf,
        key_file: Option<PathBuf>,
    ) -> Command<VaultTabMessage> {
        self.state =
            VaultTabState::Unlock(VaultUnlocker::new(crate::vault::unlock::ConstructorParam {
                path,
                key_file,
            }));
        Command::none()
    }

    /// Change the content of the tab to the [`VaultContainer`](VaultContainer).
    fn change_to_open_state(&mut self, vault: Box<Vault>) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Open(VaultContainer::new(vault));
        Command::none()
    }

    /// Change the content of the tab to the [`Settings`](Settings).
    pub fn change_to_settings_state(&mut self) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Settings(Settings::new(()));
        Command::none()
    }

    /// Update the tab state.
    fn update_state<P: Platform + 'static>(
        &mut self,
        message: VaultTabMessage,
        application_settings: &mut pwduck_core::ApplicationSettings,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultTabMessage>, PWDuckGuiError> {
        match (message, &mut self.state) {
            (VaultTabMessage::Loader(msg), VaultTabState::Empty(loader)) => Ok(loader
                .update::<P>(msg, application_settings, modal_state, clipboard)?
                .map(VaultTabMessage::Loader)),
            (VaultTabMessage::Creator(msg), VaultTabState::Create(creator)) => Ok(creator
                .update::<P>(msg, application_settings, modal_state, clipboard)?
                .map(VaultTabMessage::Creator)),
            (VaultTabMessage::Unlocker(msg), VaultTabState::Unlock(unlocker)) => Ok(unlocker
                .update::<P>(msg, application_settings, modal_state, clipboard)?
                .map(VaultTabMessage::Unlocker)),
            (VaultTabMessage::Container(msg), VaultTabState::Open(container)) => Ok(container
                .update::<P>(msg, application_settings, modal_state, clipboard)?
                .map(VaultTabMessage::Container)),
            (VaultTabMessage::Settings(msg), VaultTabState::Settings(settings)) => Ok(settings
                .update::<P>(msg, application_settings, modal_state, clipboard)?
                .map(VaultTabMessage::Settings)),
            _ => PWDuckGuiError::Unreachable("VaultTabMessage".into()).into(),
        }
    }
}

/// A vector containing all tabs and the index of the selected one.
#[derive(Debug)]
pub struct VaultTabVec(usize, Vec<VaultTab>);

impl VaultTabVec {
    /// Create a new [`VaultTabVec`](VaultTabVec) for the given list of tabs.
    #[must_use]
    pub fn new(index: usize, tabs: Vec<VaultTab>) -> Self {
        Self(index, tabs)
    }

    /// Get the currently selected tab index.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.0
    }

    /// Select a tab with the given index.
    pub fn select(&mut self, index: usize) {
        self.0 = index;
    }
}

impl std::ops::Deref for VaultTabVec {
    type Target = Vec<VaultTab>;

    #[cfg_attr(coverage, no_coverage)]
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl std::ops::DerefMut for VaultTabVec {
    #[cfg_attr(coverage, no_coverage)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl Focus for VaultTabVec {
    fn focus(&mut self, direction: iced_focus::Direction) -> iced_focus::State {
        self.1
            .get_mut(self.0)
            .map_or(iced_focus::State::Ignored, |t| t.focus(direction))
    }

    fn has_focus(&self) -> bool {
        self.1[self.0].has_focus()
    }
}

/// The message produced by the [`VaultTab`](VaultTab).
#[derive(Clone, Debug)]
pub enum VaultTabMessage {
    /// The message produced by the [`VaultLoader`](VaultLoader).
    Loader(VaultLoaderMessage),
    /// The message produced by the [`VaultCreator`](VaultCreator).
    Creator(VaultCreatorMessage),
    /// The message produced by the [`VaultContainer`](VaultContainer).
    Container(VaultContainerMessage),
    /// The message produced by the [`VaultUnlocker`](VaultUnlocker).
    Unlocker(VaultUnlockerMessage),
    /// The message produced by the [`Settings`](Settings).
    Settings(SettingsMessage),
}

/// The states of the tab content.
#[derive(Debug, Focus)]
pub enum VaultTabState {
    /// The state of the [`VaultLoader`](VaultLoader).
    Empty(#[focus(enable)] VaultLoader),
    /// The state of the [`VaultCreator`](VaultCreator).
    Create(#[focus(enable)] Box<VaultCreator>),
    /// The state of the [`VaultContainer`](VaultCreator).
    Open(#[focus(enable)] VaultContainer),
    /// The state of the [`VaultUnlocker`](VaultUnlocker).
    Unlock(#[focus(enable)] VaultUnlocker),
    /// The state of the [`Settings`](Settings).
    Settings(#[focus(enable)] Settings),
}

#[cfg_attr(test, mockable)]
impl Component for VaultTab {
    type Message = VaultTabMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {
            state: VaultTabState::Empty(VaultLoader::new(())),
        }
    }

    fn title(&self) -> String {
        match &self.state {
            VaultTabState::Empty(loader) => loader.title(),
            VaultTabState::Create(creator) => creator.title(),
            VaultTabState::Open(container) => container.title(),
            VaultTabState::Unlock(unlocker) => unlocker.title(),
            VaultTabState::Settings(settings) => settings.title(),
        }
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        application_settings: &mut pwduck_core::ApplicationSettings,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError> {
        match (message.clone(), &mut self.state) {
            // Handling Messages of sub elements.
            (VaultTabMessage::Loader(VaultLoaderMessage::Create), _) => {
                Ok(self.change_to_create_state())
            }

            (
                VaultTabMessage::Creator(VaultCreatorMessage::Cancel)
                | VaultTabMessage::Unlocker(VaultUnlockerMessage::Close),
                _,
            ) => Ok(self.change_to_empty_state()),

            (VaultTabMessage::Creator(VaultCreatorMessage::VaultCreated(vault_data)), _) => {
                let (path, key_file) = vault_data?;
                Ok(self.change_to_unlock_state(path, key_file))
            }

            (
                VaultTabMessage::Unlocker(VaultUnlockerMessage::Unlocked(vault))
                | VaultTabMessage::Loader(VaultLoaderMessage::Loaded(vault)),
                _,
            ) => Ok(self.change_to_open_state(vault?)),

            (
                VaultTabMessage::Container(VaultContainerMessage::ToolBar(
                    ToolBarMessage::LockVault,
                )),
                VaultTabState::Open(container),
            ) => {
                let path = container.vault().path().clone();
                let key_file = container.vault().key_file().clone();
                Ok(self.change_to_unlock_state(path, key_file))
            }

            // Passing every other message to sub elements
            _ => self.update_state::<P>(message, application_settings, modal_state, clipboard),
        }
    }

    #[cfg_attr(coverage, no_coverage)]
    fn view<P: Platform + 'static>(
        &mut self,
        application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message> {
        match &mut self.state {
            VaultTabState::Empty(loader) => loader
                .view::<P>(application_settings, theme, viewport)
                .map(VaultTabMessage::Loader),
            VaultTabState::Create(creator) => creator
                .view::<P>(application_settings, theme, viewport)
                .map(VaultTabMessage::Creator),
            VaultTabState::Open(container) => container
                .view::<P>(application_settings, theme, viewport)
                .map(VaultTabMessage::Container),
            VaultTabState::Unlock(unlocker) => unlocker
                .view::<P>(application_settings, theme, viewport)
                .map(VaultTabMessage::Unlocker),
            VaultTabState::Settings(settings) => settings
                .view::<P>(application_settings, theme, viewport)
                .map(VaultTabMessage::Settings),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
    };
    use tempfile::tempdir;

    use mocktopus::mocking::*;

    use iced::Command;

    use crate::{
        error::PWDuckGuiError,
        vault::{
            container::VaultContainer, creator::VaultCreator, loader::VaultLoader,
            settings::Settings, unlock::VaultUnlocker,
        },
        Component, TestPlatform,
    };

    use super::{VaultTab, VaultTabMessage, VaultTabState};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    #[test]
    fn contains_unsaved_changes() {
        let mut vault_tab = VaultTab::new(());
        assert!(!vault_tab.contains_unsaved_changes());

        let _ = vault_tab.change_to_empty_state();
        assert!(!vault_tab.contains_unsaved_changes());

        let _ = vault_tab.change_to_create_state();
        assert!(!vault_tab.contains_unsaved_changes());

        let mem_key = pwduck_core::MemKey::with_length(1);
        let password = "password";
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");
        let vault = pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
            .unwrap();
        let _ = vault_tab.change_to_open_state(Box::new(vault));

        VaultContainer::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));
        assert!(!vault_tab.contains_unsaved_changes());

        VaultContainer::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(true));
        assert!(vault_tab.contains_unsaved_changes());

        let _ = vault_tab.change_to_unlock_state(path.into(), None);
        assert!(!vault_tab.contains_unsaved_changes());

        let _ = vault_tab.change_to_settings_state();
        assert!(!vault_tab.contains_unsaved_changes());
    }

    #[test]
    fn change_to_create_state() {
        let mut vaul_tab = VaultTab::new(());
        if let VaultTabState::Create(_) = vaul_tab.state {
            panic!("VaultTab should be in the empty state");
        }

        let _ = vaul_tab.change_to_create_state();

        if let VaultTabState::Create(_) = vaul_tab.state {
        } else {
            panic!("VaultTab should be in the create state");
        }
    }

    #[test]
    fn change_to_empty_state() {
        let mut vault_tab = VaultTab::new(());
        let _ = vault_tab.change_to_create_state();
        if let VaultTabState::Empty(_) = vault_tab.state {
            panic!("VaultTab should be in the create state");
        }

        let _ = vault_tab.change_to_empty_state();

        if let VaultTabState::Empty(_) = vault_tab.state {
        } else {
            panic!("VaultTab should be in the empty state");
        }
    }

    #[test]
    fn change_to_unlock_state() {
        let mut vault_tab = VaultTab::new(());
        if let VaultTabState::Unlock(_) = vault_tab.state {
            panic!("VaultTab should be in the empty state");
        }

        let _ = vault_tab.change_to_unlock_state("path".into(), None);

        if let VaultTabState::Unlock(_) = vault_tab.state {
        } else {
            panic!("VaultTab should be in the unlock state");
        }
    }

    #[test]
    fn change_to_open_state() {
        let mut vault_tab = VaultTab::new(());
        if let VaultTabState::Open(_) = vault_tab.state {
            panic!("VaultTab should be in the empty state");
        }

        let mem_key = pwduck_core::MemKey::with_length(1);
        let password = "password";
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");
        let vault = pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
            .unwrap();
        let _ = vault_tab.change_to_open_state(Box::new(vault));

        if let VaultTabState::Open(_) = vault_tab.state {
        } else {
            panic!("VaultTab should be in the open state");
        }
    }

    #[test]
    fn change_to_settings_state() {
        let mut vault_tab = VaultTab::new(());
        if let VaultTabState::Settings(_) = vault_tab.state {
            panic!("VaultTab should be in the empty state");
        }

        let _ = vault_tab.change_to_settings_state();

        if let VaultTabState::Settings(_) = vault_tab.state {
        } else {
            panic!("VaultTab should be in the settings state");
        }
    }

    #[test]
    fn new() {
        let vault_tab = VaultTab::new(());

        if let VaultTabState::Empty(_) = vault_tab.state {
        } else {
            panic!("VaultTab should be in the empty state");
        }
    }

    #[test]
    fn title() {
        let mut vault_tab = VaultTab::new(());

        // Empty title
        if let VaultTabState::Empty(ref empty) = vault_tab.state {
            assert_eq!(empty.title(), vault_tab.title());
        } else {
            panic!("VaultTab should be empty state");
        }

        // Create title
        vault_tab.change_to_create_state();
        if let VaultTabState::Create(ref create) = vault_tab.state {
            assert_eq!(create.title(), vault_tab.title());
        } else {
            panic!("VaultTab should be create state");
        }

        // Open title
        let mem_key = pwduck_core::MemKey::with_length(1);
        let password = "password";
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");
        let vault = pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
            .unwrap();

        vault_tab.change_to_open_state(Box::new(vault));
        if let VaultTabState::Open(ref open) = vault_tab.state {
            assert_eq!(open.title(), vault_tab.title());
        } else {
            panic!("VaultTab should be open state");
        }

        // Unlock title
        vault_tab.change_to_unlock_state(path, None);
        if let VaultTabState::Unlock(ref unlock) = vault_tab.state {
            assert_eq!(unlock.title(), vault_tab.title());
        } else {
            panic!("VaultTab should be unlock state");
        }

        // Settings title
        vault_tab.change_to_settings_state();
        if let VaultTabState::Settings(ref settings) = vault_tab.state {
            assert_eq!(settings.title(), vault_tab.title());
        } else {
            panic!("VaultTab should be settings state");
        }
    }

    #[test]
    fn update() {
        let mut vault_tab = VaultTab::new(());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultTab::contains_unsaved_changes.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultTab::change_to_create_state.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultTab::change_to_empty_state.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultTab::change_to_unlock_state.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultTab::change_to_open_state.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultTab::change_to_settings_state.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultTab::update_state::<TestPlatform>.type_id(), 0);

            VaultTab::contains_unsaved_changes.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::contains_unsaved_changes.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(false)
            });
            VaultTab::change_to_create_state.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::change_to_create_state.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultTab::change_to_empty_state.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::change_to_empty_state.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultTab::change_to_unlock_state.mock_raw(|_self, _path, _key_file| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::change_to_unlock_state.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultTab::change_to_open_state.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::change_to_open_state.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultTab::change_to_settings_state.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::change_to_settings_state.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultTab::update_state::<TestPlatform>.mock_raw(|_self, _m, _a, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::update_state::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Continue((_self, _m, _a, _mod, _c))
            });

            // Change to create state
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_create_state.type_id()],
                0
            );
            let _ = vault_tab.update::<TestPlatform>(
                VaultTabMessage::Loader(crate::vault::loader::VaultLoaderMessage::Create),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_create_state.type_id()],
                1
            );

            // Change to empty state
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_empty_state.type_id()],
                0
            );
            let _ = vault_tab.update::<TestPlatform>(
                VaultTabMessage::Creator(crate::vault::creator::VaultCreatorMessage::Cancel),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_empty_state.type_id()],
                1
            );
            let _ = vault_tab.update::<TestPlatform>(
                VaultTabMessage::Unlocker(crate::vault::unlock::VaultUnlockerMessage::Close),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_empty_state.type_id()],
                2
            );

            // Change to unlock state from creator
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                0
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Creator(
                        crate::vault::creator::VaultCreatorMessage::VaultCreated(Ok((
                            "path".into(),
                            None,
                        ))),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                1
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Creator(
                        crate::vault::creator::VaultCreatorMessage::VaultCreated(Err(
                            pwduck_core::PWDuckCoreError::Error("".into()),
                        )),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                1
            );

            let mem_key = pwduck_core::MemKey::with_length(1);
            let password = "password";
            let dir = tempdir().unwrap();
            let path = dir.path().join("TempVault");
            let vault =
                pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                    .unwrap();

            // Change to open state
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_open_state.type_id()],
                0
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Unlocker(
                        crate::vault::unlock::VaultUnlockerMessage::Unlocked(Ok(Box::new(
                            vault.clone(),
                        ))),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_open_state.type_id()],
                1
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Unlocker(
                        crate::vault::unlock::VaultUnlockerMessage::Unlocked(Err(
                            pwduck_core::PWDuckCoreError::Error("".into()),
                        )),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_open_state.type_id()],
                1
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Loader(crate::vault::loader::VaultLoaderMessage::Loaded(Ok(
                        Box::new(vault.clone()),
                    ))),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_open_state.type_id()],
                2
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Loader(crate::vault::loader::VaultLoaderMessage::Loaded(Err(
                        pwduck_core::PWDuckCoreError::Error("".into()),
                    ))),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_open_state.type_id()],
                2
            );

            // Change to unlock state from open state
            vault_tab.state = VaultTabState::Open(crate::vault::container::VaultContainer::new(
                Box::new(vault),
            ));
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                1
            );
            let _ = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Container(
                        crate::vault::container::VaultContainerMessage::ToolBar(
                            crate::vault::container::ToolBarMessage::LockVault,
                        ),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                2
            );
            vault_tab.state = VaultTabState::Empty(crate::vault::loader::VaultLoader::new(()));
            assert_eq!(
                call_map.borrow()[&VaultTab::update_state::<TestPlatform>.type_id()],
                0
            );
            let res = vault_tab
                .update::<TestPlatform>(
                    VaultTabMessage::Container(
                        crate::vault::container::VaultContainerMessage::ToolBar(
                            crate::vault::container::ToolBarMessage::LockVault,
                        ),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                2
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::update_state::<TestPlatform>.type_id()],
                1
            );

            assert_eq!(
                call_map.borrow()[&VaultTab::contains_unsaved_changes.type_id()],
                0
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_create_state.type_id()],
                1
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_empty_state.type_id()],
                2
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_unlock_state.type_id()],
                2
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_open_state.type_id()],
                2
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::change_to_settings_state.type_id()],
                0
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::update_state::<TestPlatform>.type_id()],
                1
            );
        })
    }

    #[test]
    fn update_state() {
        let mut vault_tab = VaultTab::new(());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultLoader::update::<TestPlatform>.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::update::<TestPlatform>.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultUnlocker::update::<TestPlatform>.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::update::<TestPlatform>.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(Settings::update::<TestPlatform>.type_id(), 0);

            VaultLoader::update::<TestPlatform>.mock_raw(|_self, _m, _a, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultCreator::update::<TestPlatform>.mock_raw(|_self, _m, _a, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultUnlocker::update::<TestPlatform>.mock_raw(|_self, _m, _a, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultUnlocker::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::update::<TestPlatform>.mock_raw(|_self, _m, _a, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            Settings::update::<TestPlatform>.mock_raw(|_self, _m, _a, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&Settings::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Update loader
            assert_eq!(
                call_map.borrow()[&VaultLoader::update::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Loader(crate::vault::loader::VaultLoaderMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail.");
            assert_eq!(
                call_map.borrow()[&VaultLoader::update::<TestPlatform>.type_id()],
                1
            );
            // Switch to create state.
            vault_tab.change_to_create_state();
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Loader(crate::vault::loader::VaultLoaderMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            assert_eq!(
                call_map.borrow()[&VaultLoader::update::<TestPlatform>.type_id()],
                1
            );

            // Update creator
            assert_eq!(
                call_map.borrow()[&VaultCreator::update::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Creator(crate::vault::creator::VaultCreatorMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail.");
            assert_eq!(
                call_map.borrow()[&VaultCreator::update::<TestPlatform>.type_id()],
                1
            );
            // Switch to unlock state
            vault_tab.change_to_unlock_state("path".into(), None);
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Creator(crate::vault::creator::VaultCreatorMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            assert_eq!(
                call_map.borrow()[&VaultCreator::update::<TestPlatform>.type_id()],
                1
            );

            let mem_key = pwduck_core::MemKey::with_length(1);
            let password = "password";
            let dir = tempdir().unwrap();
            let path = dir.path().join("TempVault");
            let vault =
                pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                    .unwrap();

            // Update unlocker
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::update::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Unlocker(crate::vault::unlock::VaultUnlockerMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail.");
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::update::<TestPlatform>.type_id()],
                1
            );
            // Switch to open state
            vault_tab.change_to_open_state(Box::new(vault));
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Unlocker(crate::vault::unlock::VaultUnlockerMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::update::<TestPlatform>.type_id()],
                1
            );

            // Update container
            assert_eq!(
                call_map.borrow()[&VaultContainer::update::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Container(
                        crate::vault::container::VaultContainerMessage::ToolBar(
                            crate::vault::container::ToolBarMessage::LockVault,
                        ),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail.");
            assert_eq!(
                call_map.borrow()[&VaultContainer::update::<TestPlatform>.type_id()],
                1
            );
            // Switch to settings state
            vault_tab.change_to_settings_state();
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Container(
                        crate::vault::container::VaultContainerMessage::ToolBar(
                            crate::vault::container::ToolBarMessage::LockVault,
                        ),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            assert_eq!(
                call_map.borrow()[&VaultContainer::update::<TestPlatform>.type_id()],
                1
            );

            // Update settings
            assert_eq!(
                call_map.borrow()[&Settings::update::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Settings(
                        crate::vault::settings::SettingsMessage::ThemeChanged(
                            pwduck_core::theme::Theme::Dark,
                        ),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail.");
            assert_eq!(
                call_map.borrow()[&Settings::update::<TestPlatform>.type_id()],
                1
            );
            // Switch to empty state
            vault_tab.change_to_empty_state();
            let _ = vault_tab
                .update_state::<TestPlatform>(
                    VaultTabMessage::Settings(
                        crate::vault::settings::SettingsMessage::ThemeChanged(
                            pwduck_core::theme::Theme::Dark,
                        ),
                    ),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            assert_eq!(
                call_map.borrow()[&Settings::update::<TestPlatform>.type_id()],
                1
            );

            call_map.borrow().values().all(|v| *v == 1);
        });
    }
}
