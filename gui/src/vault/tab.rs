//! TODO

use iced::Command;

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
        let msg = match (message, &mut self.state) {
            // Handling Messages of sub elements.
            (VaultTabMessage::Loader(VaultLoaderMessage::Create), _) => {
                self.state = VaultTabState::Create(Box::new(VaultCreator::new(())));
                Command::none()
            }
            (VaultTabMessage::Creator(VaultCreatorMessage::Cancel), _)
            | (VaultTabMessage::Unlocker(VaultUnlockerMessage::Close), _) => {
                self.state = VaultTabState::Empty(VaultLoader::new(()));
                Command::none()
            }
            (VaultTabMessage::Creator(VaultCreatorMessage::VaultCreated(vault)), _) => {
                self.state = VaultTabState::Unlock(VaultUnlocker::new(vault?));
                Command::none()
            }
            (VaultTabMessage::Unlocker(VaultUnlockerMessage::Unlocked(vault)), _)
            | (VaultTabMessage::Loader(VaultLoaderMessage::Loaded(vault)), _) => {
                self.state = VaultTabState::Open(VaultContainer::new(vault?));
                Command::none()
            }
            (
                VaultTabMessage::Container(VaultContainerMessage::ToolBar(
                    ToolBarMessage::LockVault,
                )),
                VaultTabState::Open(container),
            ) => {
                self.state =
                    VaultTabState::Unlock(VaultUnlocker::new(container.vault().path().clone()));
                Command::none()
            }

            // Passing every other message to sub elements
            (VaultTabMessage::Loader(msg), VaultTabState::Empty(loader)) => loader
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Loader),
            (VaultTabMessage::Creator(msg), VaultTabState::Create(creator)) => creator
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Creator),
            (VaultTabMessage::Unlocker(msg), VaultTabState::Unlock(unlocker)) => unlocker
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Unlocker),
            (VaultTabMessage::Container(msg), VaultTabState::Open(container)) => container
                .update::<P>(msg, clipboard)?
                .map(VaultTabMessage::Container),
            _ => return PWDuckGuiError::Unreachable("VaultTabMessage".into()).into(),
        };
        Ok(msg)
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
