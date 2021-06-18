//! TODO

use std::path::PathBuf;

use iced::Command;
use pwduck_core::Vault;

use crate::{error::PWDuckGuiError, vault::container::ToolBarMessage, Component, Platform};

pub use super::container::VaultContainerMessage;
use super::{
    container::VaultContainer,
    creator::{VaultCreator, VaultCreatorMessage},
    loader::{VaultLoader, VaultLoaderMessage},
    unlock::{VaultUnlocker, VaultUnlockerMessage},
};

/// TODO
#[derive(Debug)]
pub struct VaultTab {
    /// TODO
    state: VaultTabState,
}

impl VaultTab {
    /// TODO
    #[must_use]
    pub fn contains_unsaved_changes(&self) -> bool {
        match &self.state {
            VaultTabState::Open(container) => container.vault().contains_unsaved_changes(),
            _ => false,
        }
    }

    /// TODO
    fn change_to_create_state(&mut self) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Create(Box::new(VaultCreator::new(())));
        Command::none()
    }

    /// TODO
    fn change_to_empty_state(&mut self) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Empty(VaultLoader::new(()));
        Command::none()
    }

    /// TODO
    fn change_to_unlock_state(&mut self, vault: PathBuf) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Unlock(VaultUnlocker::new(vault));
        Command::none()
    }

    /// TODO
    fn change_to_open_state(&mut self, vault: Box<Vault>) -> Command<VaultTabMessage> {
        self.state = VaultTabState::Open(VaultContainer::new(vault));
        Command::none()
    }

    /// TODO
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

/// TODO
#[derive(Clone, Debug)]
pub enum VaultTabMessage {
    /// TODO
    Loader(VaultLoaderMessage),
    /// TODO
    Creator(VaultCreatorMessage),
    /// TODO
    Container(VaultContainerMessage),
    /// TODO
    Unlocker(VaultUnlockerMessage),
}

/// TODO
#[derive(Debug)]
pub enum VaultTabState {
    /// TODO
    Empty(VaultLoader),
    /// TODO
    Create(Box<VaultCreator>),
    /// TODO
    Open(VaultContainer),
    /// TODO
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

            (VaultTabMessage::Creator(VaultCreatorMessage::Cancel), _)
            | (VaultTabMessage::Unlocker(VaultUnlockerMessage::Close), _) => {
                Ok(self.change_to_empty_state())
            }

            (VaultTabMessage::Creator(VaultCreatorMessage::VaultCreated(vault)), _) => {
                Ok(self.change_to_unlock_state(vault?))
            }

            (VaultTabMessage::Unlocker(VaultUnlockerMessage::Unlocked(vault)), _)
            | (VaultTabMessage::Loader(VaultLoaderMessage::Loaded(vault)), _) => {
                Ok(self.change_to_open_state(vault?))
            }

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
        //platform: &dyn Platform
    ) -> iced::Element<'_, Self::Message> {
        match &mut self.state {
            VaultTabState::Empty(loader) => loader.view::<P>().map(VaultTabMessage::Loader),
            VaultTabState::Create(creator) => creator.view::<P>().map(VaultTabMessage::Creator),
            VaultTabState::Open(container) => container.view::<P>().map(VaultTabMessage::Container),
            VaultTabState::Unlock(unlocker) => unlocker.view::<P>().map(VaultTabMessage::Unlocker),
        }
    }
}
