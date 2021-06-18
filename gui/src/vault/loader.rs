//! TODO

use std::path::PathBuf;

use iced::{button, text_input, Command, Length, Row, Text, TextInput};
use pwduck_core::{PWDuckCoreError, SecString, Vault};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    utils::{centered_container_with_column, icon_button},
    Component, Platform, DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
#[derive(Debug, Default)]
pub struct VaultLoader {
    /// TODO
    path: String,
    /// TODO
    path_state: text_input::State,

    /// TODO
    password: SecString,
    /// TODO
    password_state: text_input::State,
    /// TODO
    show_password: bool,
    /// TOOD
    show_password_state: button::State,

    /// TODO
    create_state: button::State,
    /// TODO
    confirm_state: button::State,

    /// TODO
    path_open_fd_state: button::State,
}

impl VaultLoader {
    /// TODO
    fn update_path(&mut self, path: String) -> Command<VaultLoaderMessage> {
        self.path = path;
        Command::none()
    }

    /// TODO
    fn update_password(&mut self, password: String) -> Command<VaultLoaderMessage> {
        self.password = password.into();
        Command::none()
    }

    /// TODO
    fn toggle_password_visibility(&mut self) -> Command<VaultLoaderMessage> {
        self.show_password = !self.show_password;
        Command::none()
    }

    /// TODO
    fn confirm(&mut self) -> Command<VaultLoaderMessage> {
        Command::perform(
            {
                let mut password = self.password.clone();
                self.password.zeroize();

                let path = PathBuf::from(self.path.clone());
                // TODO: remove duplicate
                async move {
                    //let mem_key = crate::MEM_KEY.lock().await;
                    let mem_key = crate::MEM_KEY.lock()?;
                    let vault = pwduck_core::Vault::load(&password, &mem_key, path);

                    //Box::new(vault)
                    vault.map(Box::new)
                }
            },
            VaultLoaderMessage::Loaded,
        )
    }

    /// TODO
    fn open_file_dialog<P: Platform + 'static>() -> Command<VaultLoaderMessage> {
        Command::perform(P::nfd_choose_folder(), VaultLoaderMessage::PathSelected)
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum VaultLoaderMessage {
    /// TODO
    PathInput(String),
    /// TODO
    PasswordInput(String),
    /// TODO
    ShowPassword,
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

            password: SecString::default(),
            password_state: text_input::State::new(),
            show_password: false,
            show_password_state: button::State::new(),

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
            VaultLoaderMessage::PathInput(input) => self.update_path(input),

            VaultLoaderMessage::PasswordInput(input) => self.update_password(input),

            VaultLoaderMessage::ShowPassword => self.toggle_password_visibility(),

            VaultLoaderMessage::Confirm => self.confirm(),

            VaultLoaderMessage::OpenFileDialog => Self::open_file_dialog::<P>(),

            VaultLoaderMessage::PathSelected(Ok(path)) => {
                self.update_path(path.to_str().ok_or(PWDuckGuiError::Option)?.to_owned())
            }

            VaultLoaderMessage::PathSelected(Err(_err)) => Command::none(),

            VaultLoaderMessage::Create | VaultLoaderMessage::Loaded(_) => {
                return PWDuckGuiError::Unreachable("VaultLoaderMessage".into()).into()
            }
        };
        Ok(cmd)
    }

    fn view<P: Platform + 'static>(
        &mut self,
        //platform: &dyn Platform
    ) -> iced::Element<'_, Self::Message> {
        let mut path_fd_button =
            icon_button(&mut self.path_open_fd_state, "I", "Open").width(Length::Shrink);
        if P::is_nfd_available() {
            path_fd_button = path_fd_button.on_press(VaultLoaderMessage::OpenFileDialog);
        }

        let vault_path = TextInput::new(
            &mut self.path_state,
            "Chose a Vault",
            &self.path,
            VaultLoaderMessage::PathInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let mut password = TextInput::new(
            &mut self.password_state,
            "Password",
            &self.password,
            VaultLoaderMessage::PasswordInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);
        if !self.show_password {
            password = password.password();
        }

        let show_password = icon_button(
            &mut self.show_password_state,
            "I",
            if self.show_password {
                // TODO
                "H"
            } else {
                "S"
            },
        )
        .width(Length::Shrink)
        .on_press(VaultLoaderMessage::ShowPassword);

        let create = icon_button(&mut self.create_state, "I", "Create new")
            .on_press(VaultLoaderMessage::Create);

        let mut unlock_vault = icon_button(&mut self.confirm_state, "I", "Unlock Vault");
        if !self.path.is_empty() && !self.password.is_empty() {
            unlock_vault = unlock_vault.on_press(VaultLoaderMessage::Confirm);
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
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(password)
                .push(show_password)
                .into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(create)
                .push(unlock_vault)
                .into(),
        ])
        .into()
    }
}
