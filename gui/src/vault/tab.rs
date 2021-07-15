//! TODO

use std::path::PathBuf;

use iced::Command;
use pwduck_core::Vault;

use crate::{
    error::PWDuckGuiError, vault::container::ToolBarMessage, Component, Platform, Viewport,
};

pub use super::container::VaultContainerMessage;
use super::{
    container::VaultContainer,
    creator::{VaultCreator, VaultCreatorMessage},
    loader::{VaultLoader, VaultLoaderMessage},
    unlock::{VaultUnlocker, VaultUnlockerMessage},
};

/// The state of a vault tab.
#[derive(Debug)]
pub struct VaultTab {
    /// The state of the tab content.
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

    /// Update the tab state.
    fn update_state<P: Platform + 'static>(
        &mut self,
        message: VaultTabMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultTabMessage>, PWDuckGuiError> {
        match (message, &mut self.state) {
            (VaultTabMessage::Loader(msg), VaultTabState::Empty(loader)) => Ok(loader
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Loader)),
            (VaultTabMessage::Creator(msg), VaultTabState::Create(creator)) => Ok(creator
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Creator)),
            (VaultTabMessage::Unlocker(msg), VaultTabState::Unlock(unlocker)) => Ok(unlocker
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Unlocker)),
            (VaultTabMessage::Container(msg), VaultTabState::Open(container)) => Ok(container
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Container)),
            _ => PWDuckGuiError::Unreachable("VaultTabMessage".into()).into(),
        }
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
}

/// The states of the tab content.
#[derive(Debug)]
pub enum VaultTabState {
    /// The state of the [`VaultLoader`](VaultLoader).
    Empty(VaultLoader),
    /// The state of the [`VaultCreator`](VaultCreator).
    Create(Box<VaultCreator>),
    /// The state of the [`VaultContainer`](VaultCreator).
    Open(VaultContainer),
    /// The state of the [`VaultUnlocker`](VaultUnlocker).
    Unlock(VaultUnlocker),
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
        }
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
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
            _ => self.update_state::<P>(message, clipboard),
        }
    }

    fn view<P: Platform + 'static>(
        &mut self,
        viewport: &Viewport,
        //platform: &dyn Platform
    ) -> iced::Element<'_, Self::Message> {
        match &mut self.state {
            VaultTabState::Empty(loader) => loader.view::<P>(viewport).map(VaultTabMessage::Loader),
            VaultTabState::Create(creator) => {
                creator.view::<P>(viewport).map(VaultTabMessage::Creator)
            }
            VaultTabState::Open(container) => container
                .view::<P>(viewport)
                .map(VaultTabMessage::Container),
            VaultTabState::Unlock(unlocker) => {
                unlocker.view::<P>(viewport).map(VaultTabMessage::Unlocker)
            }
        }
    }
}
