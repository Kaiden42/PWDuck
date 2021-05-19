//! TODO

use std::path::PathBuf;

use iced::{
    button, text_input, Button, Column, Command, Container, HorizontalAlignment, Length, Row, Text,
    TextInput,
};
use pwduck_core::{PWDuckCoreError, Vault};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    utils::centered_container_with_column,
    Component, Platform, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_HEADER_SIZE,
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
#[derive(Debug, Default)]
pub struct VaultLoader {
    /// TODO
    path: String,
    /// TODO
    path_state: text_input::State,
    /// TODO
    password: String,
    /// TODO
    password_state: text_input::State,
    /// TODO
    create_state: button::State,
    /// TODO
    confirm_state: button::State,
    /// TODO
    path_open_fd_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum VaultLoaderMessage {
    /// TODO
    PathInput(String),
    /// TODO
    PasswordInput(String),
    /// TODO
    Create,
    /// TODO
    Confirm,
    /// TODO
    OpenFileDialog,
    /// TODO
    PathSelected(Result<PathBuf, NfdError>),
    /// TODO
    Loaded(Result<Box<Vault>, PWDuckCoreError>),
}

impl Component for VaultLoader {
    type Message = VaultLoaderMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {
            path: String::new(),
            path_state: text_input::State::new(),
            password: String::new(),
            password_state: text_input::State::new(),
            create_state: button::State::new(),
            confirm_state: button::State::new(),
            path_open_fd_state: button::State::new(),
        }
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError> {
        let cmd = match message {
            VaultLoaderMessage::PathInput(input) => {
                self.path = input;
                Command::none()
            }
            VaultLoaderMessage::PasswordInput(input) => {
                self.password.zeroize();
                self.password = input;
                Command::none()
            }
            VaultLoaderMessage::Confirm => Command::perform(
                {
                    let mut password = self.password.clone();
                    self.password.zeroize();

                    let path = PathBuf::from(self.path.clone());
                    // TODO: remove duplicate
                    async move {
                        //let mem_key = crate::MEM_KEY.lock().await;
                        let mem_key = crate::MEM_KEY.lock().unwrap();
                        let vault = pwduck_core::Vault::load(&password, &mem_key, path);

                        password.zeroize();

                        //Box::new(vault)
                        vault.map(|v| Box::new(v))
                    }
                },
                VaultLoaderMessage::Loaded,
            ),
            VaultLoaderMessage::OpenFileDialog => Command::perform(
                //<platform as Platform>::open_file(),
                P::nfd_choose_folder(),
                VaultLoaderMessage::PathSelected,
            ),
            VaultLoaderMessage::PathSelected(Ok(path)) => {
                self.path = path.to_str().unwrap().to_owned();
                Command::none()
            }
            VaultLoaderMessage::PathSelected(Err(_err)) => Command::none(),
            VaultLoaderMessage::Create | VaultLoaderMessage::Loaded(_) => unreachable!(),
        };
        Ok(cmd)
    }

    fn view<P: Platform + 'static>(
        &mut self,
        //platform: &dyn Platform
    ) -> iced::Element<'_, Self::Message> {
        let mut path_fd_button = Button::new(&mut self.path_open_fd_state, Text::new("Open"));
        if P::is_nfd_available() {
            path_fd_button = path_fd_button.on_press(Self::Message::OpenFileDialog);
        }

        let vault_path = TextInput::new(
            &mut self.path_state,
            "Chose a Vault",
            &self.path,
            Self::Message::PathInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let password = TextInput::new(
            &mut self.password_state,
            "Password",
            &self.password,
            Self::Message::PasswordInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING)
        .password();

        let create = Button::new(
            &mut self.create_state,
            Text::new("Create new")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .on_press(Self::Message::Create)
        .width(Length::Fill);

        let mut unlock_vault = Button::new(
            &mut self.confirm_state,
            Text::new("Unlock Vault")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);

        if !self.path.is_empty() && !self.password.is_empty() {
            unlock_vault = unlock_vault.on_press(Self::Message::Confirm);
        }

        centered_container_with_column(vec![
            Text::new("Open existing Vault:")
                .size(DEFAULT_HEADER_SIZE)
                .into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(vault_path)
                .push(path_fd_button)
                .into(),
            password.into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(create)
                .push(unlock_vault)
                .into(),
        ])
        .into()
    }
}
