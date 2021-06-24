//! TODO

use std::path::PathBuf;

use iced::{
    button, text_input, Command, Container, Element, Row,
    Text, TextInput,
};
use pwduck_core::{PWDuckCoreError, PasswordInfo, SecString};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    icons::Icon,
    password_score::PasswordScore,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space,
        estimate_password_strength, icon_button,
    },
    Component, Platform, DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
#[derive(Debug, Default)]
pub struct VaultCreator {
    /// TODO
    name: String,
    /// TODO
    name_state: text_input::State,
    /// TODO
    path: String,
    /// TODO
    path_state: text_input::State,
    /// TODO
    path_open_fd_state: button::State,
    /// TODO
    password: SecString,
    /// TODO
    password_state: text_input::State,
    /// TODO
    password_show: bool,
    /// TODO
    password_show_state: button::State,
    /// TODO
    password_confirm: SecString,
    /// TODO
    password_confirm_state: text_input::State,
    /// TODO
    password_confirm_show: bool,
    /// tODO
    password_confirm_show_state: button::State,
    /// TODO
    password_equal: bool,
    /// TODO
    password_score: Option<PasswordScore>,
    /// TODO
    cancel_state: button::State,
    /// TODO
    submit_state: button::State,
}

impl VaultCreator {
    /// TODO
    fn update_name(&mut self, name: String) -> Command<VaultCreatorMessage> {
        self.name = name;
        Command::none()
    }

    /// TODO
    fn update_path(&mut self, path: String) -> Command<VaultCreatorMessage> {
        self.path = path;
        Command::none()
    }

    /// TODO
    fn update_password(&mut self, password: String) -> Command<VaultCreatorMessage> {
        self.password = password.into();
        self.check_password_equality();
        self.estimate_password_strength()
    }

    /// TODO
    fn toggle_password_visibility(&mut self) -> Command<VaultCreatorMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// TODO
    fn update_password_confirm(&mut self, password: String) -> Command<VaultCreatorMessage> {
        self.password_confirm = password.into();
        self.check_password_equality();
        Command::none()
    }

    /// TODO
    fn toggle_password_confirm_visibility(&mut self) -> Command<VaultCreatorMessage> {
        self.password_confirm_show = !self.password_confirm_show;
        Command::none()
    }

    /// TODO
    fn check_password_equality(&mut self) {
        self.password_equal = !self.password.is_empty() && self.password == self.password_confirm;
    }

    /// TODO
    fn estimate_password_strength(&self) -> Command<VaultCreatorMessage> {
        Command::perform(
            estimate_password_strength(self.password.clone()),
            VaultCreatorMessage::PasswordScore,
        )
    }

    fn set_password_score(
        &mut self,
        password_info: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<VaultCreatorMessage> {
        self.password_score = Some(PasswordScore::new(password_info));
        Command::none()
    }

    /// TODO
    fn submit(&mut self) -> Command<VaultCreatorMessage> {
        Command::perform(
            {
                let password = self.password.clone();
                self.password.zeroize();
                self.password_confirm.zeroize();

                //let path = self.path.clone();
                let path: PathBuf = self.path.clone().into();
                let path = path.join(self.name.clone());
                async move {
                    //let mem_key = crate::MEM_KEY.lock().await;
                    let mem_key = crate::MEM_KEY.lock().unwrap();
                    let mut vault = pwduck_core::Vault::generate(&password, &mem_key, path)?;

                    vault.save(&mem_key)?;

                    Ok(vault.path().clone())
                }
            },
            VaultCreatorMessage::VaultCreated,
        )
    }

    /// TODO
    fn open_file_dialog<P: Platform + 'static>() -> Command<VaultCreatorMessage> {
        Command::perform(P::nfd_choose_folder(), VaultCreatorMessage::PathSelected)
    }
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
    PasswordShow,
    /// TODO
    PathSelected(Result<PathBuf, NfdError>),
    /// TODO
    PasswordConfirmInput(String),
    /// TODO
    PasswordConfirmShow,
    /// TODO
    PasswordScore(Result<PasswordInfo, PWDuckCoreError>),
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
    ) -> Result<Command<Self::Message>, PWDuckGuiError> {
        let cmd = match message {
            VaultCreatorMessage::NameInput(input) => self.update_name(input),

            VaultCreatorMessage::PathInput(input) => self.update_path(input),

            VaultCreatorMessage::PathOpenFD => Self::open_file_dialog::<P>(),

            VaultCreatorMessage::PathSelected(Ok(path)) => {
                self.update_path(path.to_str().ok_or(PWDuckGuiError::Option)?.to_owned())
            }

            VaultCreatorMessage::PathSelected(Err(_err)) => Command::none(),

            VaultCreatorMessage::PasswordInput(input) => self.update_password(input),

            VaultCreatorMessage::PasswordShow => self.toggle_password_visibility(),

            VaultCreatorMessage::PasswordConfirmInput(input) => self.update_password_confirm(input),

            VaultCreatorMessage::PasswordConfirmShow => self.toggle_password_confirm_visibility(),

            VaultCreatorMessage::Submit => self.submit(),

            VaultCreatorMessage::PasswordScore(password_info) => {
                self.set_password_score(password_info)
            }

            VaultCreatorMessage::Cancel | VaultCreatorMessage::VaultCreated(_) => {
                return PWDuckGuiError::Unreachable("VaultCreatorMessage".into()).into()
            }
        };
        Ok(cmd)
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

        let mut path_fd_button = icon_button(
            &mut self.path_open_fd_state,
            Icon::Folder,
            "Open",
            "Choose the location to store your new Vault",
            true,
        );
        if P::is_nfd_available() {
            path_fd_button = path_fd_button.on_press(Self::Message::PathOpenFD);
        }

        let mut password = default_text_input(
            &mut self.password_state,
            "Enter your password",
            &self.password,
            VaultCreatorMessage::PasswordInput,
        );
        if !self.password_show {
            password = password.password();
        }

        let password_show = if self.password_show {
            icon_button(
                &mut self.password_show_state,
                Icon::EyeSlash,
                "Hide password",
                "Hide password",
                true,
            )
        } else {
            icon_button(
                &mut self.password_show_state,
                Icon::Eye,
                "Show password",
                "Show password",
                true,
            )
        }
        .on_press(VaultCreatorMessage::PasswordShow);

        let mut password_confirm = default_text_input(
            &mut self.password_confirm_state,
            "Confirm your password",
            &self.password_confirm,
            VaultCreatorMessage::PasswordConfirmInput,
        );
        if !self.password_confirm_show {
            password_confirm = password_confirm.password();
        }

        let password_confirm_show = if self.password_confirm_show {
            icon_button(
                &mut self.password_confirm_show_state,
                Icon::EyeSlash,
                "Hide password",
                "Hide password",
                true,
            )
        } else {
            icon_button(
                &mut self.password_confirm_show_state,
                Icon::Eye,
                "Show password",
                "Show password",
                true,
            )
        }
        .on_press(VaultCreatorMessage::PasswordConfirmShow);

        let password_score: Element<_> = self.password_score.as_mut().map_or_else(
            || Container::new(default_vertical_space()).into(),
            PasswordScore::view,
        );

        if !self.password.is_empty() && !self.password_equal {
            password_confirm = password_confirm.style(PasswordNotEqualStyle)
        }

        let cancel_button = icon_button(
            &mut self.cancel_state,
            Icon::XSquare,
            "Cancel",
            "Cancel creation of new Vault",
            false,
        )
        .on_press(Self::Message::Cancel);

        let mut submit_button = icon_button(
            &mut self.submit_state,
            Icon::Save,
            "Submit",
            "Submit creation of new Vault",
            false,
        );
        if self.password_equal
            && !self.password.is_empty()
            && !self.name.is_empty()
            && !self.path.is_empty()
        {
            submit_button = submit_button.on_press(Self::Message::Submit)
        }

        centered_container_with_column(vec![
            Text::new("Create a new Vault:")
                .size(DEFAULT_HEADER_SIZE)
                .into(),
            name.into(),
            default_vertical_space().into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(path)
                .push(path_fd_button)
                .into(),
            default_vertical_space().into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(password)
                .push(password_show)
                .into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(password_confirm)
                .push(password_confirm_show)
                .into(),
            password_score,
            default_vertical_space().into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(cancel_button)
                .push(submit_button)
                .into(),
        ])
        .into()
    }
}

/// TODO
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
