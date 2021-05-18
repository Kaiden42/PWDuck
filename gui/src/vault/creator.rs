//! TODO

use std::{path::PathBuf, sync::Arc};

use iced::{
    button,
    futures::lock::Mutex,
    text_input::{self, StyleSheet},
    Button, Column, Command, Container, HorizontalAlignment, Length, Row, Space, Text, TextInput,
};
use zeroize::Zeroize;

use crate::{
    error::NfdError, Component, Platform, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING,
    DEFAULT_HEADER_SIZE, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT,
    DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
#[derive(Debug, Default)]
pub struct VaultCreator {
    name: String,
    name_state: text_input::State,
    path: String,
    path_state: text_input::State,
    path_open_fd_state: button::State,
    password: String,
    password_state: text_input::State,
    password_confirm: String,
    password_confirm_state: text_input::State,
    password_equal: bool,
    cancel_state: button::State,
    submit_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum VaultCreatorMessage {
    /// TODO
    NameInput(String),
    /// TODO
    PathInput(String),
    /// TODO
    PathOpenFD,
    /// TODO
    PasswordInput(String),
    /// TODO
    PathSelected(Result<PathBuf, NfdError>),
    /// TODO
    PasswordConfirmInput(String),
    /// TODO
    Cancel,
    /// TODO
    Submit,
    /// TODO
    VaultCreated(Result<PathBuf, pwduck_core::PWDuckCoreError>),
}

impl Component for VaultCreator {
    type Message = VaultCreatorMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self::default()
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match message {
            VaultCreatorMessage::NameInput(input) => {
                self.name = input;
                Command::none()
            }
            VaultCreatorMessage::PathInput(input) => {
                self.path = input;
                Command::none()
            }
            VaultCreatorMessage::PathOpenFD => {
                Command::perform(P::nfd_choose_folder(), VaultCreatorMessage::PathSelected)
            }
            VaultCreatorMessage::PathSelected(Ok(path)) => {
                self.path = path.to_str().unwrap().to_owned();
                Command::none()
            }
            VaultCreatorMessage::PathSelected(Err(_err)) => Command::none(),
            VaultCreatorMessage::PasswordInput(input) => {
                self.password.zeroize();
                self.password = input;
                self.password_equal =
                    !self.password.is_empty() && self.password == self.password_confirm;
                Command::none()
            }
            VaultCreatorMessage::PasswordConfirmInput(input) => {
                self.password_confirm.zeroize();
                self.password_confirm = input;
                self.password_equal =
                    !self.password_confirm.is_empty() && self.password == self.password_confirm;
                Command::none()
            }
            VaultCreatorMessage::Cancel => unreachable!(),
            VaultCreatorMessage::Submit => {
                Command::perform(
                    {
                        let mut password = self.password.clone();
                        self.password.zeroize();
                        self.password_confirm.zeroize();

                        //let path = self.path.clone();
                        let path: PathBuf = self.path.clone().into();
                        let path = path.join(self.name.clone());
                        async move {
                            //let mem_key = crate::MEM_KEY.lock().await;
                            let mem_key = crate::MEM_KEY.lock().unwrap();
                            let mut vault =
                                pwduck_core::Vault::generate(&password, &mem_key, path)?;
                            password.zeroize();

                            vault.save(&mem_key)?;

                            Ok(vault.path().to_owned())
                        }
                    },
                    VaultCreatorMessage::VaultCreated,
                )
            }
            VaultCreatorMessage::VaultCreated(_) => unreachable!(),
        }
    }

    fn view<P: Platform + 'static>(&mut self) -> iced::Element<'_, Self::Message> {
        let name = TextInput::new(
            &mut self.name_state,
            "Enter the name of your vault",
            &self.name,
            Self::Message::NameInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let path = TextInput::new(
            &mut self.path_state,
            "Choose the location for your vault",
            &self.path,
            Self::Message::PathInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let mut path_fd_button = Button::new(&mut self.path_open_fd_state, Text::new("Open"));
        if P::is_nfd_available() {
            path_fd_button = path_fd_button.on_press(Self::Message::PathOpenFD);
        }

        let password = TextInput::new(
            &mut self.password_state,
            "Enter your password",
            &self.password,
            Self::Message::PasswordInput,
        )
        .password()
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let mut password_confirm = TextInput::new(
            &mut self.password_confirm_state,
            "Confirm your password",
            &self.password_confirm,
            Self::Message::PasswordConfirmInput,
        )
        .password()
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        if !self.password.is_empty() && !self.password_equal {
            password_confirm = password_confirm.style(PasswordNotEqualStyle)
        }

        let cancel_button = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .on_press(Self::Message::Cancel)
        .width(Length::Fill);

        let mut submit_button = Button::new(
            &mut self.submit_state,
            Text::new("Submit")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);
        if self.password_equal
            && !self.password.is_empty()
            && !self.name.is_empty()
            && !self.path.is_empty()
        {
            submit_button = submit_button.on_press(Self::Message::Submit)
        }

        Container::new(
            Column::new()
                .max_width(DEFAULT_MAX_WIDTH)
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_COLUMN_SPACING)
                .push(Text::new("Create a new Vault:").size(DEFAULT_HEADER_SIZE))
                .push(name)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(path)
                        .push(path_fd_button),
                )
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(password)
                .push(password_confirm)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(cancel_button)
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

#[derive(Default)]
struct PasswordNotEqualStyle;

impl text_input::StyleSheet for PasswordNotEqualStyle {
    fn active(&self) -> text_input::Style {
        use iced::{Background, Color};
        text_input::Style {
            background: Background::Color(Color::WHITE),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::from_rgb(1.0, 0.3, 0.3),
        }
    }

    fn focused(&self) -> text_input::Style {
        use iced::Color;
        text_input::Style {
            border_color: Color::from_rgb(1.0, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> iced::Color {
        iced::Color::from_rgb(1.0, 0.3, 0.3)
    }

    fn value_color(&self) -> iced::Color {
        iced::Color::from_rgb(1.0, 0.3, 0.3)
    }

    fn selection_color(&self) -> iced::Color {
        iced::Color::from_rgb(1.0, 0.8, 0.8)
    }
}
