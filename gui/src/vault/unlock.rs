//! TODO

use std::path::PathBuf;

use iced::{
    button, text_input, Button, Column, Command, Container, HorizontalAlignment, Length, Row,
    Space, Text, TextInput,
};
use pwduck_core::{PWDuckCoreError, SecString, Vault};
use zeroize::Zeroize;

use crate::{
    error::PWDuckGuiError, Component, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING,
    DEFAULT_HEADER_SIZE, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT,
    DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
#[derive(Debug, Default)]
pub struct VaultUnlocker {
    /// TODO
    path: PathBuf,
    /// TODO
    password: SecString,
    /// TODO
    password_state: text_input::State,
    /// TODO
    close_state: button::State,
    /// TODO
    submit_state: button::State,
}

impl VaultUnlocker {
    /// TODO
    fn update_password(&mut self, password: String) -> Command<VaultUnlockerMessage> {
        self.password = password.into();
        Command::none()
    }

    /// TODO
    fn submit(&mut self) -> Command<VaultUnlockerMessage> {
        Command::perform(
            {
                let password = self.password.clone();
                self.password.zeroize();
                let path = self.path.clone();
                async move {
                    let mem_key = crate::MEM_KEY.lock()?;
                    let vault = pwduck_core::Vault::load(&password, &mem_key, path);

                    vault.map(Box::new)
                }
            },
            VaultUnlockerMessage::Unlocked,
        )
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum VaultUnlockerMessage {
    /// TODO
    PasswordInput(String),
    /// TODO
    Close,
    /// TODO
    Submit,
    /// TODO
    Unlocked(Result<Box<Vault>, PWDuckCoreError>),
}

impl Component for VaultUnlocker {
    type Message = VaultUnlockerMessage;
    type ConstructorParam = PathBuf;

    fn new(path: Self::ConstructorParam) -> Self {
        //Self { ..Self::default() }
        Self {
            path,
            ..Self::default()
        }
    }

    fn update<P: crate::Platform + 'static>(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError> {
        match message {
            VaultUnlockerMessage::PasswordInput(password) => Ok(self.update_password(password)),
            VaultUnlockerMessage::Submit => Ok(self.submit()),
            VaultUnlockerMessage::Close | VaultUnlockerMessage::Unlocked(_) => {
                PWDuckGuiError::Unreachable("VaultUnlockerMessage".into()).into()
            }
        }
    }

    fn view<P: crate::Platform + 'static>(&mut self) -> iced::Element<'_, Self::Message> {
        let path = PathBuf::from(&self.path);
        let vault_name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("Name of Vault");

        let path = Text::new(self.path.to_str().unwrap_or("Invalid path"));

        let password = TextInput::new(
            &mut self.password_state,
            "Enter password to unlock",
            &self.password,
            Self::Message::PasswordInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING)
        .password();

        let close_button = Button::new(
            &mut self.close_state,
            Text::new("Close")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .on_press(Self::Message::Close)
        .width(Length::Fill);

        let mut submit_button = Button::new(
            &mut self.submit_state,
            Text::new("Unlock")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);

        if !self.password.is_empty() {
            submit_button = submit_button.on_press(Self::Message::Submit);
        }

        Container::new(
            Column::new()
                .max_width(DEFAULT_MAX_WIDTH)
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_COLUMN_SPACING)
                .push(Text::new(&format!("Unlock vault: {}", vault_name)).size(DEFAULT_HEADER_SIZE))
                .push(path)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(password)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(close_button)
                        .push(submit_button),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
