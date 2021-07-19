//! TODO

use std::path::PathBuf;

use iced::{button, text_input, Command, Container, Element, Row, Text};
use pwduck_core::{PWDuckCoreError, PasswordInfo, SecString};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    icons::Icon,
    password_score::PasswordScore,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space,
        estimate_password_strength, icon_button, password_toggle, SomeIf,
    },
    Component, Platform, Viewport, DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING,
};

/// The state of the vault creator.
#[derive(Debug, Default)]
pub struct VaultCreator {
    /// The name of the new vault.
    name: String,
    /// The state of the [`TextInput`](iced::TextInput) for the name.
    name_state: text_input::State,
    /// The location of the new vault.
    path: String,
    /// The state of the [`TextInput`](iced::TextInput) for the location.
    path_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to open the native file dialog.
    path_open_fd_state: button::State,
    /// The password of the new vault.
    password: SecString,
    /// The state of the [`TextInput`](iced::TextInput) for the password.
    password_state: text_input::State,
    /// The visibility of the password.
    password_show: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility of the password.
    password_show_state: button::State,
    /// The confirmation of the password.
    password_confirm: SecString,
    /// The state of the [`TextInput`](iced::TextInput) for the password confirmation.
    password_confirm_state: text_input::State,
    /// The visibility of the password confirmation.
    password_confirm_show: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility of the password confirmation.
    password_confirm_show_state: button::State,
    /// If the password equals the password confirmation.
    password_equal: bool,
    /// The estimated score of the password.
    password_score: Option<PasswordScore>,
    /// The state of the cancel [`Button`](iced::Button).
    cancel_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,
}

impl VaultCreator {
    /// Update the name and replace it with the given value.
    fn update_name(&mut self, name: String) -> Command<VaultCreatorMessage> {
        self.name = name;
        Command::none()
    }

    /// Update the path and replace it with the given value.
    fn update_path(&mut self, path: String) -> Command<VaultCreatorMessage> {
        self.path = path;
        Command::none()
    }

    /// Update the password and replace it with the given value.
    fn update_password(&mut self, password: String) -> Command<VaultCreatorMessage> {
        self.password = password.into();
        self.check_password_equality();
        self.estimate_password_strength()
    }

    /// Toggle the visibility of the password.
    fn toggle_password_visibility(&mut self) -> Command<VaultCreatorMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// Update the password confirmation and replace it with the given value.
    fn update_password_confirm(&mut self, password: String) -> Command<VaultCreatorMessage> {
        self.password_confirm = password.into();
        self.check_password_equality();
        Command::none()
    }

    /// Toggle the visibility of the password confirmation.
    fn toggle_password_confirm_visibility(&mut self) -> Command<VaultCreatorMessage> {
        self.password_confirm_show = !self.password_confirm_show;
        Command::none()
    }

    /// Check if the password equals the password confirmation.
    fn check_password_equality(&mut self) {
        self.password_equal = !self.password.is_empty() && self.password == self.password_confirm;
    }

    /// Estimate the strength of the password.
    fn estimate_password_strength(&self) -> Command<VaultCreatorMessage> {
        Command::perform(
            estimate_password_strength(self.password.clone()),
            VaultCreatorMessage::PasswordScore,
        )
    }

    /// Set the estimated score of the password.
    fn set_password_score(
        &mut self,
        password_info: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<VaultCreatorMessage> {
        self.password_score = Some(PasswordScore::new(password_info));
        Command::none()
    }

    /// Submit the creation of the new vault.
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

    /// Open the native file dialog of the [`Platform`](Platform).
    fn open_file_dialog<P: Platform + 'static>() -> Command<VaultCreatorMessage> {
        Command::perform(P::nfd_choose_folder(), VaultCreatorMessage::PathSelected)
    }
}

/// The message that is send by the vault creator.
#[derive(Clone, Debug)]
pub enum VaultCreatorMessage {
    /// Change the name to the new value.
    NameInput(String),
    /// Change the path to the new value.
    PathInput(String),
    /// Open the native file dialog.
    PathOpenFD,
    /// Change the password to the new value.
    PasswordInput(String),
    /// Toggle the visibility of the password.
    PasswordShow,
    /// The path was selected by the native file dialog.
    PathSelected(Result<PathBuf, NfdError>),
    /// Change the password confirmation to the new value.
    PasswordConfirmInput(String),
    /// Toggle the visibility of the password.
    PasswordConfirmShow,
    /// Set the password score to the new estimated value.
    PasswordScore(Result<PasswordInfo, PWDuckCoreError>),
    /// Cancel the creation of the new vault.
    Cancel,
    /// Submit the creation of the new vault.
    Submit,
    /// The vault was successfully created.
    VaultCreated(Result<PathBuf, pwduck_core::PWDuckCoreError>),
}
impl SomeIf for VaultCreatorMessage {}

impl Component for VaultCreator {
    type Message = VaultCreatorMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        "Create a new vault".into()
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _modal_state: &mut iced_aw::modal::State<crate::ModalState>,
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

    fn view<P: Platform + 'static>(
        &mut self,
        _viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message> {
        let name = default_text_input(
            &mut self.name_state,
            "Enter the name of your new vault",
            &self.name,
            VaultCreatorMessage::NameInput,
        );

        let path_row = path_row::<P>(
            &mut self.path_state,
            &self.path,
            &mut self.path_open_fd_state,
        );

        let password_row = password_row(
            &mut self.password_state,
            &self.password,
            self.password_show,
            &mut self.password_show_state,
        );

        let password_confirm_row = password_confirm_row(
            &mut self.password_confirm_state,
            &self.password_confirm,
            self.password_confirm_show,
            &mut self.password_confirm_show_state,
            self.password.is_empty(),
            self.password_equal,
        );

        let password_score: Element<_> = self.password_score.as_mut().map_or_else(
            || Container::new(default_vertical_space()).into(),
            PasswordScore::view,
        );

        let button_row = button_row(
            &mut self.cancel_state,
            &mut self.submit_state,
            self.password_equal
                && !self.password.is_empty()
                && !self.name.is_empty()
                && !self.path.is_empty(),
        );

        centered_container_with_column(vec![
            Text::new("Create a new Vault:")
                .size(DEFAULT_HEADER_SIZE)
                .into(),
            name.into(),
            default_vertical_space().into(),
            path_row,
            default_vertical_space().into(),
            password_row,
            password_confirm_row,
            password_score,
            default_vertical_space().into(),
            button_row,
        ])
        .into()
    }
}

/// Create the view of the path selection.
///
/// It expects:
///     - The state of the [`TextInput`](iced::TextInput)
///     - The value of the path
///     - The state of the [`Button`](iced::Button) to open the native file dialog
fn path_row<'a, P: Platform + 'static>(
    path_state: &'a mut text_input::State,
    path: &'a str,
    path_open_fd_state: &'a mut button::State,
) -> Element<'a, VaultCreatorMessage> {
    let path = default_text_input(
        path_state,
        "Choose the location for your new vault",
        path,
        VaultCreatorMessage::PathInput,
    );

    let path_fd_button = icon_button(
        path_open_fd_state,
        Icon::Folder,
        "Open",
        "Choose the location to store your new Vault",
        true,
        VaultCreatorMessage::PathOpenFD.some_if(P::is_nfd_available()),
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(path)
        .push(path_fd_button)
        .into()
}

/// Create the view of the password selection.
///
/// It expects:
///     - The state of the [`TextInput`](iced::TextInput)
///     - The value of the password
///     - The visibility of the password
///     - The state of the [`Button`](iced::Button) to toggle the visibility
fn password_row<'a>(
    password_state: &'a mut text_input::State,
    password: &'a str,
    password_show: bool,
    password_show_state: &'a mut button::State,
) -> Element<'a, VaultCreatorMessage> {
    let mut password = default_text_input(
        password_state,
        "Enter your password",
        password,
        VaultCreatorMessage::PasswordInput,
    );
    if !password_show {
        password = password.password();
    }

    let password_show = password_toggle(
        password_show_state,
        password_show,
        VaultCreatorMessage::PasswordShow,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(password)
        .push(password_show)
        .into()
}

/// Create the view of the password confirmation selection.
///
/// It expects:
///     - The state of the [`TextInput`](TextInput)
///     - The value of the password confirmation
///     - The visibility of the password confirmation
///     - The state of the [`Button`] to toggle the visibility
///     - True, if the password is empty
///     - True, if the password equals the password confirmation
fn password_confirm_row<'a>(
    password_confirm_state: &'a mut text_input::State,
    password_confirm: &str,
    password_confirm_show: bool,
    password_confirm_show_state: &'a mut button::State,
    password_empty: bool,
    password_equal: bool,
) -> Element<'a, VaultCreatorMessage> {
    let mut password_confirm = default_text_input(
        password_confirm_state,
        "Confirm your password",
        password_confirm,
        VaultCreatorMessage::PasswordConfirmInput,
    );
    if !password_confirm_show {
        password_confirm = password_confirm.password();
    }

    let password_confirm_show = password_toggle(
        password_confirm_show_state,
        password_confirm_show,
        VaultCreatorMessage::PasswordConfirmShow,
    );

    if !password_empty && !password_equal {
        password_confirm = password_confirm.style(PasswordNotEqualStyle)
    }

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(password_confirm)
        .push(password_confirm_show)
        .into()
}

/// Create the view of the submit and cancel button.
///
/// It expects:
///     - The state of the cancel [`Button`](iced::Button)
///     - The state of the submit [`Button`](iced::Button)
///     - True, if the creation can be submitted.
fn button_row<'a>(
    cancel_state: &'a mut button::State,
    submit_state: &'a mut button::State,
    can_submit: bool,
) -> Element<'a, VaultCreatorMessage> {
    let cancel_button = icon_button(
        cancel_state,
        Icon::XSquare,
        "Cancel",
        "Cancel creation of new Vault",
        false,
        Some(VaultCreatorMessage::Cancel),
    );

    let submit_button = icon_button(
        submit_state,
        Icon::Save,
        "Submit",
        "Submit creation of new Vault",
        false,
        VaultCreatorMessage::Submit.some_if(can_submit),
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(cancel_button)
        .push(submit_button)
        .into()
}

/// The style of the password confirmation if the passwords are not equal.
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
