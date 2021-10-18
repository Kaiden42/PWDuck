//! TODO

use iced::{Column, Command, Element};

use crate::error::PWDuckGuiError;

/// TODO
#[derive(Debug, Default)]
pub struct PassphraseTabState {}

/// TODO
#[derive(Clone, Debug)]
pub enum PassphraseTabMessage {
    /// TODO
    Todo,
}

impl PassphraseTabState {
    /// TODO
    pub fn update(
        &mut self,
        _message: &PassphraseTabMessage,
    ) -> Result<Command<PassphraseTabMessage>, PWDuckGuiError> {
        Ok(Command::none())
    }

    /// TODO
    pub fn view(&mut self) -> Element<PassphraseTabMessage> {
        // TODO
        Column::new().into()
    }

    /// TODO
    pub fn generate(&self) -> String {
        // TODO
        String::new()
    }
}
