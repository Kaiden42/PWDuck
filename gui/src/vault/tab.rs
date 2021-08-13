//! TODO

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

/// The state of a vault tab.
#[derive(Debug, Focus)]
pub struct VaultTab {
    /// The state of the tab content.
    #[focus(enable)]
    state: VaultTabState,
}

impl VaultTab {
    /// True, if the tab contains unsaved changes.
    #[must_use]
    pub fn contains_unsaved_changes(&self) -> bool {
        match &self.state {
            VaultTabState::Open(container) => {
                container.vault().contains_unsaved_changes()
                    || container
                        .modify_group_view()
                        .as_ref()
                        //.map(|view| view.group().is_modified())
                        //.unwrap_or(false)
                        .map_or(false, |view| view.group().is_modified())
                    || container
                        .modify_entry_view()
                        .as_ref()
                        //.map(|view| {
                        //    view.entry_head().is_modified() || view.entry_body().is_modified()
                        //})
                        //.unwrap_or(false)
                        .map_or(false, |view| {
                            view.entry_head().is_modified() || view.entry_body().is_modified()
                        })
            }
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
    fn change_to_unlock_state(&mut self, vault: PathBuf) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Unlock(VaultUnlocker::new(vault));
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

/// TODO
#[derive(Debug)]
pub struct VaultTabVec(usize, Vec<VaultTab>);

impl VaultTabVec {
    /// TODO
    pub fn new(index: usize, tabs: Vec<VaultTab>) -> Self {
        Self (index, tabs)
    }

    /// TODO
    pub fn selected(&self) -> usize {
        self.0
    }

    /// TODO
    pub fn select(&mut self, index: usize) {
        self.0 = index;
    }
}

impl std::ops::Deref for VaultTabVec {
    type Target = Vec<VaultTab>;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl std::ops::DerefMut for VaultTabVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl Focus for VaultTabVec {
    fn focus(&mut self, direction: iced_focus::Direction) -> iced_focus::State {
        self.1.get_mut(self.0).map_or(iced_focus::State::Ignored, |t| t.focus(direction))
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
    Empty(
        #[focus(enable)]
        VaultLoader
    ),
    /// The state of the [`VaultCreator`](VaultCreator).
    Create(
        #[focus(enable)]
        Box<VaultCreator>
    ),
    /// The state of the [`VaultContainer`](VaultCreator).
    Open(
        #[focus(enable)]
        VaultContainer
    ),
    /// The state of the [`VaultUnlocker`](VaultUnlocker).
    Unlock(
        #[focus(enable)]
        VaultUnlocker
    ),
    /// The state of the [`Settings`](Settings).
    Settings(
        #[focus(enable)]
        Settings
    ),
}

impl Component for VaultTab {
    type Message = VaultTabMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {
            state: VaultTabState::Empty(VaultLoader::new(())),
            //state: VaultTabState::Unlock(VaultUnlocker::with_path("/home/robert/Schreibtisch/TestVault".into())),
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

            (VaultTabMessage::Creator(VaultCreatorMessage::VaultCreated(vault)), _) => {
                Ok(self.change_to_unlock_state(vault?))
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
                Ok(self.change_to_unlock_state(path))
            }

            // Passing every other message to sub elements
            _ => self.update_state::<P>(message, application_settings, modal_state, clipboard),
        }
    }

    fn view<P: Platform + 'static>(
        &mut self,
        application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        viewport: &Viewport,
        //platform: &dyn Platform
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
