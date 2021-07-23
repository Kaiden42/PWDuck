//! TODO

use std::path::PathBuf;

use iced::{button, text_input, Command, Row, Text};
use pwduck_core::{PWDuckCoreError, SecString, Vault};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    icons::Icon,
    theme::Theme,
    utils::{
        centered_container_with_column, default_text_input, icon_button, password_toggle,
        ButtonData, ButtonKind, SomeIf,
    },
    Component, Platform, Viewport, DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING,
};

/// The state of the vault loader.
#[derive(Debug, Default)]
pub struct VaultLoader {
    /// The path of the vault to load.
    path: String,
    /// The state of the [`TextInput`](iced::TextInput) of the path.
    path_state: text_input::State,

    /// The password of the vault.
    password: SecString,
    /// The state of the [`TextInput`](iced::TextInput) of the password.
    password_state: text_input::State,
    /// The visibility of the password.
    show_password: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility.
    show_password_state: button::State,

    /// The state of the [`Button`](iced::Button) to show the vault creator.
    create_state: button::State,
    /// The state of teh [`Button`](iced::Button) to confirm the loading of the vault.
    confirm_state: button::State,

    /// The state of the [`Button`](iced::Button) to open the native file dialog.
    path_open_fd_state: button::State,
}

impl VaultLoader {
    /// Update the path and replace it by the new value.
    fn update_path(&mut self, path: String) -> Command<VaultLoaderMessage> {
        self.path = path;
        Command::none()
    }

    /// Update the password and replace it by the new value.
    fn update_password(&mut self, password: String) -> Command<VaultLoaderMessage> {
        self.password = password.into();
        Command::none()
    }

    /// Toggle the visibility of the password.
    fn toggle_password_visibility(&mut self) -> Command<VaultLoaderMessage> {
        self.show_password = !self.show_password;
        Command::none()
    }

    /// Confirm the loading of the vault.
    fn confirm(&mut self) -> Command<VaultLoaderMessage> {
        Command::perform(
            {
                let password = self.password.clone();
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

    /// Open the native file dialog of the [`Platform`](Platform).
    fn open_file_dialog<P: Platform + 'static>() -> Command<VaultLoaderMessage> {
        Command::perform(P::nfd_choose_folder(), VaultLoaderMessage::PathSelected)
    }
}

/// The message created by the vault loader.
#[derive(Clone, Debug)]
pub enum VaultLoaderMessage {
    /// Change the path to the new value.
    PathInput(String),
    /// Change the password to the new value.
    PasswordInput(String),
    /// Toggle the visibility of the password.
    ShowPassword,
    /// Show the vault creator.
    Create,
    /// Confirm the loading of the vault.
    Confirm,
    /// Open the native file dialog.
    OpenFileDialog,
    /// The path was selected by the native file dialog.
    PathSelected(Result<PathBuf, NfdError>),
    /// The vault was loaded successfully.
    Loaded(Result<Box<Vault>, PWDuckCoreError>),
}
impl SomeIf for VaultLoaderMessage {}

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

    fn title(&self) -> String {
        "Load vault from storage".into()
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _application_settings: &mut pwduck_core::ApplicationSettings,
        _modal_state: &mut iced_aw::modal::State<crate::ModalState>,
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
        _application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        _viewport: &Viewport,
        //platform: &dyn Platform
    ) -> iced::Element<'_, Self::Message> {
        let path_fd_button = icon_button(
            ButtonData {
                state: &mut self.path_open_fd_state,
                icon: Icon::Folder,
                text: "Open",
                kind: ButtonKind::Normal,
                on_press: VaultLoaderMessage::OpenFileDialog.some_if(P::is_nfd_available()),
            },
            "Select the directory of the vault",
            true,
            theme,
        );

        let vault_path = default_text_input(
            &mut self.path_state,
            "Choose a Vault",
            &self.path,
            VaultLoaderMessage::PathInput,
        )
        .style(theme.text_input());

        let mut password = default_text_input(
            &mut self.password_state,
            "Password",
            &self.password,
            VaultLoaderMessage::PasswordInput,
        )
        .style(theme.text_input());
        if !self.show_password {
            password = password.password();
        }

        let show_password = password_toggle(
            &mut self.show_password_state,
            self.show_password,
            VaultLoaderMessage::ShowPassword,
            theme,
        );

        let create = icon_button(
            ButtonData {
                state: &mut self.create_state,
                icon: Icon::Safe,
                text: "Create new",
                kind: ButtonKind::Normal,
                on_press: Some(VaultLoaderMessage::Create),
            },
            "Create a new vault",
            false,
            theme,
        );

        let unlock_vault = icon_button(
            ButtonData {
                state: &mut self.confirm_state,
                icon: Icon::Unlock,
                text: "Unlock",
                kind: ButtonKind::Primary,
                on_press: VaultLoaderMessage::Confirm
                    .some_if_not(self.path.is_empty() || self.password.is_empty()),
            },
            "Unlock vault",
            false,
            theme,
        );

        centered_container_with_column(
            vec![
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
            ],
            theme,
        )
        .into()
    }
}
