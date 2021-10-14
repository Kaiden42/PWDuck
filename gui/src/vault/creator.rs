//! TODO

use std::path::PathBuf;

use iced::{button, text_input, Command, Container, Element, Row, Text};
use iced_focus::Focus;
use pwduck_core::{PWDuckCoreError, PasswordInfo, SecString};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    icons::Icon,
    password_score::PasswordScore,
    theme::Theme,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space,
        estimate_password_strength, icon_button, password_toggle, ButtonData, ButtonKind, SomeIf,
    },
    Component, Platform, Viewport, DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING,
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the vault creator.
#[derive(Debug, Default, Focus)]
pub struct VaultCreator {
    /// The name of the new vault.
    name: String,
    /// The state of the [`TextInput`](iced::TextInput) for the name.
    #[focus(enable)]
    name_state: text_input::State,
    /// The location of the new vault.
    path: String,
    /// The state of the [`TextInput`](iced::TextInput) for the location.
    #[focus(enable)]
    path_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to open the native file dialog.
    path_open_fd_state: button::State,
    /// The password of the new vault.
    password: SecString,
    /// The state of the [`TextInput`](iced::TextInput) for the password.
    #[focus(enable)]
    password_state: text_input::State,
    /// The visibility of the password.
    password_show: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility of the password.
    password_show_state: button::State,
    /// The confirmation of the password.
    password_confirm: SecString,
    /// The state of the [`TextInput`](iced::TextInput) for the password confirmation.
    #[focus(enable)]
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

#[cfg_attr(test, mockable)]
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
        if self.name.is_empty()
            || self.path.is_empty()
            || self.password.is_empty()
            || self.password_confirm.is_empty()
            || !self.password_equal
        {
            return Command::none();
        }

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

#[cfg_attr(test, mockable)]
impl Component for VaultCreator {
    type Message = VaultCreatorMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {
            name_state: text_input::State::focused(),
            ..Self::default()
        }
    }

    #[cfg_attr(coverage, no_coverage)]
    fn title(&self) -> String {
        "Create a new vault".into()
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _application_settings: &mut pwduck_core::ApplicationSettings,
        _modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<Self::Message>, PWDuckGuiError> {
        let cmd = match message {
            VaultCreatorMessage::NameInput(input) => self.update_name(input),

            VaultCreatorMessage::PathInput(input) => self.update_path(input),

            VaultCreatorMessage::PathOpenFD => Self::open_file_dialog::<P>(),

            VaultCreatorMessage::PathSelected(Ok(path)) => {
                let cmd = self.update_path(path.to_str().ok_or(PWDuckGuiError::Option)?.to_owned());
                self.path_state.unfocus();
                self.password_state.focus();
                cmd
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

    #[cfg_attr(coverage, no_coverage)]
    fn view<P: Platform + 'static>(
        &mut self,
        _application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        _viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message> {
        let name = default_text_input(
            &mut self.name_state,
            "Enter the name of your new vault",
            &self.name,
            VaultCreatorMessage::NameInput,
        )
        .on_submit(VaultCreatorMessage::Submit)
        .style(theme.text_input());

        let path_row = path_row::<P>(
            &mut self.path_state,
            &self.path,
            &mut self.path_open_fd_state,
            theme,
        );

        let password_row = password_row(
            &mut self.password_state,
            &self.password,
            self.password_show,
            &mut self.password_show_state,
            theme,
        );

        let password_confirm_row = password_confirm_row(
            &mut self.password_confirm_state,
            &self.password_confirm,
            self.password_confirm_show,
            &mut self.password_confirm_show_state,
            self.password.is_empty(),
            self.password_equal,
            theme,
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
            theme,
        );

        centered_container_with_column(
            vec![
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
            ],
            theme,
        )
        .into()
    }
}

/// Create the view of the path selection.
///
/// It expects:
///     - The state of the [`TextInput`](iced::TextInput)
///     - The value of the path
///     - The state of the [`Button`](iced::Button) to open the native file dialog
#[cfg_attr(coverage, no_coverage)]
fn path_row<'a, P: Platform + 'static>(
    path_state: &'a mut text_input::State,
    path: &'a str,
    path_open_fd_state: &'a mut button::State,
    theme: &dyn Theme,
) -> Element<'a, VaultCreatorMessage> {
    let mut path = default_text_input(
        path_state,
        "Choose the location for your new vault",
        path,
        VaultCreatorMessage::PathInput,
    )
    .style(theme.text_input());
    if P::is_nfd_available() {
        path = path.on_submit(VaultCreatorMessage::PathOpenFD);
    }

    let path_fd_button = icon_button(
        ButtonData {
            state: path_open_fd_state,
            icon: Icon::Folder,
            text: "Open",
            kind: ButtonKind::Normal,
            on_press: VaultCreatorMessage::PathOpenFD.some_if(P::is_nfd_available()),
        },
        "Choose the location to store your new vault",
        true,
        theme,
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
#[cfg_attr(coverage, no_coverage)]
fn password_row<'a>(
    password_state: &'a mut text_input::State,
    password: &'a str,
    password_show: bool,
    password_show_state: &'a mut button::State,
    theme: &dyn Theme,
) -> Element<'a, VaultCreatorMessage> {
    let mut password = default_text_input(
        password_state,
        "Enter your password",
        password,
        VaultCreatorMessage::PasswordInput,
    )
    .on_submit(VaultCreatorMessage::Submit)
    .style(theme.text_input());
    if !password_show {
        password = password.password();
    }

    let password_show = password_toggle(
        password_show_state,
        password_show,
        VaultCreatorMessage::PasswordShow,
        theme,
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
#[cfg_attr(coverage, no_coverage)]
fn password_confirm_row<'a>(
    password_confirm_state: &'a mut text_input::State,
    password_confirm: &str,
    password_confirm_show: bool,
    password_confirm_show_state: &'a mut button::State,
    password_empty: bool,
    password_equal: bool,
    theme: &dyn Theme,
) -> Element<'a, VaultCreatorMessage> {
    let mut password_confirm = default_text_input(
        password_confirm_state,
        "Confirm your password",
        password_confirm,
        VaultCreatorMessage::PasswordConfirmInput,
    )
    .on_submit(VaultCreatorMessage::Submit);
    if !password_confirm_show {
        password_confirm = password_confirm.password();
    }

    let password_confirm_show = password_toggle(
        password_confirm_show_state,
        password_confirm_show,
        VaultCreatorMessage::PasswordConfirmShow,
        theme,
    );

    password_confirm = password_confirm.style(if !password_empty && !password_equal {
        theme.password_missmatch()
    } else {
        theme.text_input()
    });

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
#[cfg_attr(coverage, no_coverage)]
fn button_row<'a>(
    cancel_state: &'a mut button::State,
    submit_state: &'a mut button::State,
    can_submit: bool,
    theme: &dyn Theme,
) -> Element<'a, VaultCreatorMessage> {
    let cancel_button = icon_button(
        ButtonData {
            state: cancel_state,
            icon: Icon::XSquare,
            text: "Cancel",
            kind: ButtonKind::Normal,
            on_press: Some(VaultCreatorMessage::Cancel),
        },
        "Cancel the creation of a new vault",
        false,
        theme,
    );

    let submit_button = icon_button(
        ButtonData {
            state: submit_state,
            icon: Icon::Save,
            text: "Submit",
            kind: ButtonKind::Primary,
            on_press: VaultCreatorMessage::Submit.some_if(can_submit),
        },
        "Submit the creation of a new vault",
        false,
        theme,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(cancel_button)
        .push(submit_button)
        .into()
}

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
    };

    use iced::Command;
    use mocktopus::mocking::*;
    use pwduck_core::{PasswordInfo, SecString};

    use crate::{
        error::{self, PWDuckGuiError},
        Component, TestPlatform,
    };

    use super::{VaultCreator, VaultCreatorMessage};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    #[test]
    fn update_name() {
        let mut vault_creator = VaultCreator::new(());
        assert!(vault_creator.name.is_empty());

        let _cmd = vault_creator.update_name("name".into());

        assert!(!vault_creator.name.is_empty());
        assert_eq!(vault_creator.name.as_str(), "name");
    }

    #[test]
    fn update_path() {
        let mut vault_creator = VaultCreator::new(());
        assert!(vault_creator.path.is_empty());

        let _cmd = vault_creator.update_path("this/is/a/path".into());

        assert!(!vault_creator.path.is_empty());
        assert_eq!(vault_creator.path.as_str(), "this/is/a/path");
    }

    #[test]
    fn update_password() {
        let mut vault_creator = VaultCreator::new(());
        assert!(vault_creator.password.is_empty());

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultCreator::check_password_equality.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::estimate_password_strength.type_id(), 0);

            VaultCreator::check_password_equality.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::check_password_equality.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });
            VaultCreator::estimate_password_strength.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::estimate_password_strength.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            let _ = vault_creator.update_password("password".into());
            assert_eq!(vault_creator.password.as_str(), "password");
            assert_eq!(
                call_map.borrow()[&VaultCreator::check_password_equality.type_id()],
                1
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::estimate_password_strength.type_id()],
                1
            );
        })
    }

    #[test]
    fn toggle_password_visibility() {
        let mut vault_creator = VaultCreator::new(());
        assert!(!vault_creator.password_show);

        let _cmd = vault_creator.toggle_password_visibility();

        assert!(vault_creator.password_show);

        let _cmd = vault_creator.toggle_password_visibility();

        assert!(!vault_creator.password_show);
    }

    #[test]
    fn update_password_confirm() {
        let mut vault_creator = VaultCreator::new(());
        assert!(vault_creator.password_confirm.is_empty());

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultCreator::check_password_equality.type_id(), 0);

            VaultCreator::check_password_equality.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::check_password_equality.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            let _ = vault_creator.update_password_confirm("password".into());
            assert_eq!(vault_creator.password_confirm.as_str(), "password");
            assert_eq!(
                call_map.borrow()[&VaultCreator::check_password_equality.type_id()],
                1
            );
        });
    }

    #[test]
    fn toggle_password_confirm_visibility() {
        let mut vault_creator = VaultCreator::new(());
        assert!(!vault_creator.password_confirm_show);

        let _cmd = vault_creator.toggle_password_confirm_visibility();

        assert!(vault_creator.password_confirm_show);

        let _cmd = vault_creator.toggle_password_confirm_visibility();

        assert!(!vault_creator.password_confirm_show);
    }

    #[test]
    fn check_password_equality() {
        let mut vault_creator = VaultCreator::new(());
        assert!(!vault_creator.password_equal);

        vault_creator.check_password_equality();
        assert!(!vault_creator.password_equal);

        vault_creator.password = SecString::from("password");
        vault_creator.check_password_equality();
        assert!(!vault_creator.password_equal);

        vault_creator.password_confirm = SecString::from("password");
        vault_creator.check_password_equality();
        assert!(vault_creator.password_equal);

        vault_creator.password = SecString::from("not password");
        vault_creator.check_password_equality();
        assert!(!vault_creator.password_equal);
    }

    #[test]
    fn estimate_password_strength() {
        let vault_creator = VaultCreator::new(());

        let cmd = vault_creator.estimate_password_strength();
        assert_eq!(cmd.futures().len(), 1);
    }

    #[test]
    fn set_password_score() {
        let mut vault_creator = VaultCreator::new(());
        assert!(vault_creator.password_score.is_none());

        let password_score = Ok(PasswordInfo::for_password("password"));
        let _ = vault_creator.set_password_score(password_score);
        assert!(vault_creator.password_score.is_some());
    }

    #[test]
    fn submit() {
        let mut vault_creator = VaultCreator::new(());

        let cmd = vault_creator.submit();
        assert!(cmd.futures().is_empty());

        let _ = vault_creator.update_name("name".into());
        let cmd = vault_creator.submit();
        assert!(cmd.futures().is_empty());

        let _ = vault_creator.update_path("this/is/a/path".into());
        let cmd = vault_creator.submit();
        assert!(cmd.futures().is_empty());

        let _ = vault_creator.update_password("password".into());
        let cmd = vault_creator.submit();
        assert!(cmd.futures().is_empty());

        let _ = vault_creator.update_password_confirm("not password".into());
        let cmd = vault_creator.submit();
        assert!(cmd.futures().is_empty());

        let _ = vault_creator.update_password_confirm("password".into());
        let cmd = vault_creator.submit();
        assert!(!cmd.futures().is_empty());
    }

    #[test]
    fn open_file_dialog() {
        let cmd = VaultCreator::open_file_dialog::<TestPlatform>();
        assert!(!cmd.futures().is_empty());
    }

    #[test]
    fn new() {
        let vault_creator = VaultCreator::new(());

        assert!(vault_creator.name.is_empty());
        assert!(vault_creator.name_state.is_focused());
        assert!(vault_creator.path.is_empty());
        assert!(!vault_creator.path_state.is_focused());
        assert!(vault_creator.password.is_empty());
        assert!(!vault_creator.password_state.is_focused());
        assert!(!vault_creator.password_show);
        assert!(vault_creator.password_confirm.is_empty());
        assert!(!vault_creator.password_confirm_state.is_focused());
        assert!(!vault_creator.password_confirm_show);
        assert!(!vault_creator.password_equal);
        assert!(vault_creator.password_score.is_none());
    }

    #[test]
    fn update() {
        let mut vault_creator = VaultCreator::new(());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultCreator::update_name.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::update_path.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::update_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::toggle_password_visibility.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::update_password_confirm.type_id(), 0);
            call_map.borrow_mut().insert(
                VaultCreator::toggle_password_confirm_visibility.type_id(),
                0,
            );
            call_map
                .borrow_mut()
                .insert(VaultCreator::set_password_score.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::submit.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultCreator::open_file_dialog::<TestPlatform>.type_id(), 0);

            VaultCreator::update_name.mock_raw(|_self, _name| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::update_name.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::update_path.mock_raw(|_self, _path| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::update_path.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::update_password.mock_raw(|_self, _password| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::update_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::toggle_password_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::toggle_password_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::update_password_confirm.mock_raw(|_self, _password| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::update_password_confirm.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::toggle_password_confirm_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::toggle_password_confirm_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::set_password_score.mock_raw(|_self, _score| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::set_password_score.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::submit.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::submit.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultCreator::open_file_dialog::<TestPlatform>.mock_raw(|| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultCreator::open_file_dialog::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Update name
            assert_eq!(call_map.borrow()[&VaultCreator::update_name.type_id()], 0);
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::NameInput("Name".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultCreator::update_name.type_id()], 1);

            // Update path
            assert_eq!(call_map.borrow()[&VaultCreator::update_path.type_id()], 0);
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PathInput("Path".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultCreator::update_path.type_id()], 1);

            // Open File Dialog
            assert_eq!(
                call_map.borrow()[&VaultCreator::open_file_dialog::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PathOpenFD,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::open_file_dialog::<TestPlatform>.type_id()],
                1
            );

            // Path selected
            assert_eq!(call_map.borrow()[&VaultCreator::update_path.type_id()], 1);
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PathSelected(Ok("path".into())),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultCreator::update_path.type_id()], 2);
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PathSelected(Err(error::NfdError::Null)),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultCreator::update_path.type_id()], 2);

            // Update password
            assert_eq!(
                call_map.borrow()[&VaultCreator::update_password.type_id()],
                0
            );
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PasswordInput("password".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::update_password.type_id()],
                1
            );

            // Toggle password visibility
            assert_eq!(
                call_map.borrow()[&VaultCreator::toggle_password_visibility.type_id()],
                0
            );
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PasswordShow,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::toggle_password_visibility.type_id()],
                1
            );

            // Update password confirm
            assert_eq!(
                call_map.borrow()[&VaultCreator::update_password_confirm.type_id()],
                0
            );
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PasswordConfirmInput("password".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::update_password_confirm.type_id()],
                1
            );

            // Toggle password confirm visibility
            assert_eq!(
                call_map.borrow()[&VaultCreator::toggle_password_confirm_visibility.type_id()],
                0
            );
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PasswordConfirmShow,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::toggle_password_confirm_visibility.type_id()],
                1
            );

            // Submit
            assert_eq!(call_map.borrow()[&VaultCreator::submit.type_id()], 0);
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::Submit,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultCreator::submit.type_id()], 1);

            // Set password score
            assert_eq!(
                call_map.borrow()[&VaultCreator::set_password_score.type_id()],
                0
            );
            let _ = vault_creator.update::<TestPlatform>(
                VaultCreatorMessage::PasswordScore(Err(pwduck_core::PWDuckCoreError::Error(
                    "".into(),
                ))),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultCreator::set_password_score.type_id()],
                1
            );

            // Cancel
            let res = vault_creator
                .update::<TestPlatform>(
                    VaultCreatorMessage::Cancel,
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            // Vault created
            let res = vault_creator
                .update::<TestPlatform>(
                    VaultCreatorMessage::VaultCreated(Err(pwduck_core::PWDuckCoreError::Error(
                        "".into(),
                    ))),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            assert!(call_map
                .borrow()
                .iter()
                .filter(|(k, _)| *k != &VaultCreator::update_path.type_id())
                .all(|(_, v)| *v == 1));
            assert_eq!(call_map.borrow()[&VaultCreator::update_path.type_id()], 2);
        });
    }
}
