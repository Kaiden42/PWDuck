//! The view of the vault loader
use std::path::PathBuf;

use iced::{button, text_input, Checkbox, Column, Command, Element, Row, Text};
use iced_focus::Focus;
use pwduck_core::{PWDuckCoreError, SecString, Vault};
use zeroize::Zeroize;

use crate::{
    error::{NfdError, PWDuckGuiError},
    icons::Icon,
    theme::Theme,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space, icon_button,
        password_toggle, ButtonData, ButtonKind, SomeIf,
    },
    Component, Platform, Viewport, DEFAULT_COLUMN_SPACING, DEFAULT_HEADER_SIZE,
    DEFAULT_ROW_SPACING,
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the vault loader.
#[derive(Debug, Default, Focus)]
pub struct VaultLoader {
    /// The path of the vault to load.
    path: String,
    /// The state of the [`TextInput`](iced::TextInput) of the path.
    #[focus(enable)]
    path_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to open the native file dialog.
    path_open_fd_state: button::State,

    /// The password of the vault.
    password: SecString,
    /// The state of the [`TextInput`](iced::TextInput) of the password.
    #[focus(enable)]
    password_state: text_input::State,
    /// The visibility of the password.
    show_password: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility.
    show_password_state: button::State,

    /// The path of the key file.
    key_file: String,
    /// The state of the [`TextInput`](iced::TextInput) of the key file path.
    key_file_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to open the native file dialog of the key file.
    key_file_open_fd_state: button::State,
    /// Use a key file
    use_key_file: bool,

    /// The state of the [`Button`](iced::Button) to show the vault creator.
    create_state: button::State,
    /// The state of teh [`Button`](iced::Button) to submit the loading of the vault.
    submit_state: button::State,
}

#[cfg_attr(test, mockable)]
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

    /// Update the key file and replace it by the new value.
    fn update_key_file(&mut self, key_file: String) -> Command<VaultLoaderMessage> {
        self.key_file = key_file;
        Command::none()
    }

    /// Toggle the usage of the key file.
    fn toggle_use_key_file(&mut self, usage: bool) -> Command<VaultLoaderMessage> {
        self.use_key_file = usage;
        if usage {
            self.key_file_state.focus();
        }
        Command::none()
    }

    /// Submit the loading of the vault.
    fn submit(&mut self) -> Command<VaultLoaderMessage> {
        if self.path.is_empty() || self.password.is_empty() {
            return Command::none();
        }

        Command::perform(
            {
                let password = self.password.clone();
                self.password.zeroize();

                let path = PathBuf::from(self.path.clone());

                let key_file = if self.use_key_file {
                    Some(self.key_file.clone())
                } else {
                    None
                };

                async move {
                    let mem_key = crate::MEM_KEY.lock()?;
                    let vault = pwduck_core::Vault::load(&password, key_file, &mem_key, path);

                    vault.map(Box::new)
                }
            },
            VaultLoaderMessage::Loaded,
        )
    }

    /// Open the native file dialog of the [`Platform`](Platform).
    fn open_file_dialog_path<P: Platform + 'static>() -> Command<VaultLoaderMessage> {
        Command::perform(P::nfd_choose_folder(), VaultLoaderMessage::PathSelected)
    }

    /// Open the native file dialog of the [`Platform`](Platform) to choose the path of the key file.
    fn open_file_dialog_key_file<P: Platform + 'static>() -> Command<VaultLoaderMessage> {
        Command::perform(
            P::nfd_choose_key_file(None),
            VaultLoaderMessage::KeyFileSelected,
        )
    }
}

/// The message created by the vault loader.
#[derive(Clone, Debug)]
pub enum VaultLoaderMessage {
    /// Change the path to the new value.
    PathInput(String),
    /// Open the native file dialog.
    OpenFileDialog,
    /// The path was selected by the native file dialog.
    PathSelected(Result<PathBuf, NfdError>),

    /// Change the password to the new value.
    PasswordInput(String),
    /// Toggle the visibility of the password.
    ShowPassword,

    /// Change the key file to the new value.
    KeyFileInput(String),
    /// Toggle the usage of a key file.
    ToggleUseKeyFile(bool),
    /// Open the native file dialog.
    KeyFileOpenFD,
    /// The path to the key file was selected by the native file dialog.
    KeyFileSelected(Result<PathBuf, NfdError>),

    /// Show the vault creator.
    Create,
    /// Submit the loading of the vault.
    Submit,

    /// The vault was loaded successfully.
    Loaded(Result<Box<Vault>, PWDuckCoreError>),
}
impl SomeIf for VaultLoaderMessage {}

#[cfg_attr(test, mockable)]
impl Component for VaultLoader {
    type Message = VaultLoaderMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {
            path: String::new(),
            path_state: text_input::State::focused(),
            path_open_fd_state: button::State::new(),

            password: SecString::default(),
            password_state: text_input::State::new(),
            show_password: false,
            show_password_state: button::State::new(),

            key_file: String::new(),
            key_file_state: text_input::State::new(),
            key_file_open_fd_state: button::State::new(),
            use_key_file: false,

            create_state: button::State::new(),
            submit_state: button::State::new(),
        }
    }

    #[cfg_attr(coverage, no_coverage)]
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

            VaultLoaderMessage::OpenFileDialog => Self::open_file_dialog_path::<P>(),

            VaultLoaderMessage::PathSelected(Ok(path)) => {
                let cmd = self.update_path(path.to_str().ok_or(PWDuckGuiError::Option)?.to_owned());
                self.path_state.unfocus();
                self.password_state.focus();
                cmd
            }

            VaultLoaderMessage::PathSelected(Err(_err)) => Command::none(),

            VaultLoaderMessage::PasswordInput(input) => self.update_password(input),

            VaultLoaderMessage::ShowPassword => self.toggle_password_visibility(),

            VaultLoaderMessage::KeyFileInput(input) => self.update_key_file(input),

            VaultLoaderMessage::ToggleUseKeyFile(b) => self.toggle_use_key_file(b),

            VaultLoaderMessage::KeyFileOpenFD => Self::open_file_dialog_key_file::<P>(),

            VaultLoaderMessage::KeyFileSelected(Ok(path)) => {
                let cmd =
                    self.update_key_file(path.to_str().ok_or(PWDuckGuiError::Option)?.to_owned());
                cmd
            }

            VaultLoaderMessage::KeyFileSelected(Err(_err)) => Command::none(),

            VaultLoaderMessage::Submit => self.submit(),

            VaultLoaderMessage::Create | VaultLoaderMessage::Loaded(_) => {
                return PWDuckGuiError::Unreachable("VaultLoaderMessage".into()).into()
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
        let path_row = path_row::<P>(
            &mut self.path_state,
            &self.path,
            &mut self.path_open_fd_state,
            theme,
        );

        let password_row = password_row(
            &mut self.password_state,
            &self.password,
            self.show_password,
            &mut self.show_password_state,
            theme,
        );

        let key_file_row = key_file_row::<P>(
            &mut self.key_file_state,
            &self.key_file,
            &mut self.key_file_open_fd_state,
            self.use_key_file,
            theme,
        );

        let button_row = button_row(
            &mut self.create_state,
            &mut self.submit_state,
            !(self.path.is_empty() || self.password.is_empty()),
            theme,
        );

        centered_container_with_column(
            vec![
                Text::new("Open existing Vault:")
                    .size(DEFAULT_HEADER_SIZE)
                    .into(),
                path_row,
                password_row,
                default_vertical_space().into(),
                key_file_row,
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
///  - The state of the [`TextInput`](iced::TextInput)
///  - The value of the path
///  - The state of the [`Button`](iced::Button) to open the native file dialog
#[cfg_attr(coverage, no_coverage)]
fn path_row<'a, P: Platform + 'static>(
    path_state: &'a mut text_input::State,
    path: &'a str,
    path_open_fd_state: &'a mut button::State,
    theme: &dyn Theme,
) -> Element<'a, VaultLoaderMessage> {
    let path_fd_button = icon_button(
        ButtonData {
            state: path_open_fd_state,
            icon: Icon::Folder,
            text: "Open",
            kind: ButtonKind::Normal,
            on_press: VaultLoaderMessage::OpenFileDialog.some_if(P::is_nfd_available()),
        },
        "Select the directory of the vault",
        true,
        theme,
    );

    let mut vault_path = default_text_input(
        path_state,
        "Choose a Vault",
        path,
        VaultLoaderMessage::PathInput,
    )
    .style(theme.text_input());
    if P::is_nfd_available() {
        vault_path = vault_path.on_submit(VaultLoaderMessage::OpenFileDialog);
    }

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(vault_path)
        .push(path_fd_button)
        .into()
}

/// Create the view of the password selection.
///
/// It expects:
///  - The state of the [`TextInput`](iced::TextInput)
///  - The value of the password
///  - The visibility of the password
///  - The state of the [`Button`](iced::Button) to toggle the visibility
#[cfg_attr(coverage, no_coverage)]
fn password_row<'a>(
    password_state: &'a mut text_input::State,
    password: &'a str,
    show_password: bool,
    show_password_state: &'a mut button::State,
    theme: &dyn Theme,
) -> Element<'a, VaultLoaderMessage> {
    let mut password = default_text_input(
        password_state,
        "Password",
        password,
        VaultLoaderMessage::PasswordInput,
    )
    .on_submit(VaultLoaderMessage::Submit)
    .style(theme.text_input());
    if !show_password {
        password = password.password();
    }

    let show_password = password_toggle(
        show_password_state,
        show_password,
        VaultLoaderMessage::ShowPassword,
        theme,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(password)
        .push(show_password)
        .into()
}

/// Create the view of the key file row.
///
/// It expects:
///  - The state of the [`TextInput`](iced::TextInput)
///  - The value of the key file path
///  - The state of the [`Button`](iced::Button) to open the native file dialog
///  - If a key file is used
#[cfg_attr(coverage, no_coverage)]
fn key_file_row<'a, P: Platform + 'static>(
    key_file_state: &'a mut text_input::State,
    key_file: &'a str,
    key_file_open_fd_state: &'a mut button::State,
    use_key_file: bool,
    theme: &dyn Theme,
) -> Element<'a, VaultLoaderMessage> {
    let check_box = Checkbox::new(
        use_key_file,
        "Select a key file as 2nd factor",
        VaultLoaderMessage::ToggleUseKeyFile,
    )
    .style(theme.checkbox());

    if !use_key_file {
        return Column::new()
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(check_box)
            .into();
    }

    let mut key_file = default_text_input(
        key_file_state,
        "Choose the location for your existing key file",
        key_file,
        VaultLoaderMessage::KeyFileInput,
    )
    .style(theme.text_input());
    if P::is_nfd_available() {
        key_file = key_file.on_submit(VaultLoaderMessage::KeyFileOpenFD);
    }

    let key_file_fd_button = icon_button(
        ButtonData {
            state: key_file_open_fd_state,
            icon: Icon::Folder,
            text: "Open",
            kind: ButtonKind::Normal,
            on_press: VaultLoaderMessage::KeyFileOpenFD.some_if(P::is_nfd_available()),
        },
        "Choose the location of the key file that belongs to the vault you want to unlock",
        true,
        theme,
    );

    let row = Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(key_file)
        .push(key_file_fd_button);

    Column::new()
        .spacing(DEFAULT_COLUMN_SPACING)
        .push(check_box)
        .push(row)
        .into()
}

/// Create the view of the submit and cancel button.
///
/// It expects:
///  - The state of the create [`Button`](iced::Button)
///  - The state of the submit [`Button`](iced::Button)
///  - True, if the creation can be submitted.
fn button_row<'a>(
    create_state: &'a mut button::State,
    submit_state: &'a mut button::State,
    can_submit: bool,
    theme: &dyn Theme,
) -> Element<'a, VaultLoaderMessage> {
    let create = icon_button(
        ButtonData {
            state: create_state,
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
            state: submit_state,
            icon: Icon::Unlock,
            text: "Unlock",
            kind: ButtonKind::Primary,
            on_press: VaultLoaderMessage::Submit.some_if(can_submit),
        },
        "Unlock vault",
        false,
        theme,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(create)
        .push(unlock_vault)
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

    use crate::{
        error::{self, PWDuckGuiError},
        Component, TestPlatform,
    };

    use super::{VaultLoader, VaultLoaderMessage};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    #[test]
    fn update_path() {
        let mut vault_loader = VaultLoader::new(());
        assert!(vault_loader.path.is_empty());

        let _cmd = vault_loader.update_path("path".into());

        assert!(!vault_loader.path.is_empty());
        assert_eq!(vault_loader.path.as_str(), "path");
    }

    #[test]
    fn update_password() {
        let mut vault_loader = VaultLoader::new(());
        assert!(vault_loader.password.is_empty());

        let _cmd = vault_loader.update_password("password".into());

        assert!(!vault_loader.password.is_empty());
        assert_eq!(vault_loader.password.as_str(), "password");
    }

    #[test]
    fn toggle_password_visibility() {
        let mut vault_loader = VaultLoader::new(());
        assert!(!vault_loader.show_password);

        let _cmd = vault_loader.toggle_password_visibility();

        assert!(vault_loader.show_password);

        let _cmd = vault_loader.toggle_password_visibility();

        assert!(!vault_loader.show_password);
    }

    #[test]
    fn update_key_file() {
        let mut vault_loader = VaultLoader::new(());
        assert!(vault_loader.key_file.is_empty());

        let _ = vault_loader.update_key_file("key_file".into());
        assert_eq!(vault_loader.key_file.as_str(), "key_file");
    }

    #[test]
    fn toggle_use_key_file() {
        let mut vault_loader = VaultLoader::new(());

        assert!(!vault_loader.use_key_file);

        let _ = vault_loader.toggle_use_key_file(true);

        assert!(vault_loader.use_key_file);

        let _ = vault_loader.toggle_use_key_file(false);

        assert!(!vault_loader.use_key_file);
    }

    #[test]
    fn submit() {
        let mut vault_loader = VaultLoader::new(());
        assert!(vault_loader.path.is_empty());
        assert!(vault_loader.password.is_empty());

        let cmd = vault_loader.submit();
        assert!(cmd.futures().is_empty());

        let _cmd = vault_loader.update_password("password".into());
        let cmd = vault_loader.submit();
        assert!(cmd.futures().is_empty());

        let _cmd = vault_loader.update_path("path".into());
        let cmd = vault_loader.submit();
        assert!(!cmd.futures().is_empty());

        // Password should be zeroized.
        let cmd = vault_loader.submit();
        assert!(cmd.futures().is_empty());
    }

    #[test]
    fn open_file_dialog() {
        let cmd = VaultLoader::open_file_dialog_path::<TestPlatform>();
        assert!(!cmd.futures().is_empty());
    }

    #[test]
    fn open_file_dialog_key_file() {
        let cmd = VaultLoader::open_file_dialog_key_file::<TestPlatform>();
        assert!(!cmd.futures().is_empty());
    }

    #[test]
    fn new() {
        let vault_loader = VaultLoader::new(());

        assert!(vault_loader.path.is_empty());
        assert!(vault_loader.path_state.is_focused());
        assert!(vault_loader.password.is_empty());
        assert!(!vault_loader.password_state.is_focused());
        assert!(!vault_loader.show_password);
    }

    #[test]
    fn update() {
        let mut vault_loader = VaultLoader::new(());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultLoader::update_path.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultLoader::update_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultLoader::toggle_password_visibility.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultLoader::update_key_file.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultLoader::toggle_use_key_file.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultLoader::submit.type_id(), 0);
            call_map.borrow_mut().insert(
                VaultLoader::open_file_dialog_path::<TestPlatform>.type_id(),
                0,
            );
            call_map.borrow_mut().insert(
                VaultLoader::open_file_dialog_key_file::<TestPlatform>.type_id(),
                0,
            );

            VaultLoader::update_path.mock_raw(|_self, _path| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::update_path.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::update_password.mock_raw(|_self, _password| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::update_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::toggle_password_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::toggle_password_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::update_key_file.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::update_key_file.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::toggle_use_key_file.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::toggle_use_key_file.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::submit.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::submit.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::open_file_dialog_path::<TestPlatform>.mock_raw(|| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::open_file_dialog_path::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::open_file_dialog_key_file::<TestPlatform>.mock_raw(|| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::open_file_dialog_key_file::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Update path
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 0);
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::PathInput("path".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 1);

            // Update password
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_password.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::PasswordInput("password".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_password.type_id()],
                1
            );

            // Toggle password visibility
            assert_eq!(
                call_map.borrow()[&VaultLoader::toggle_password_visibility.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::ShowPassword,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::toggle_password_visibility.type_id()],
                1
            );

            // Update key file
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_key_file.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::KeyFileInput("key".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_key_file.type_id()],
                1
            );

            // Toggle the usage of the key file
            assert_eq!(
                call_map.borrow()[&VaultLoader::toggle_use_key_file.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::ToggleUseKeyFile(true),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::toggle_use_key_file.type_id()],
                1
            );

            // Confirm
            assert_eq!(call_map.borrow()[&VaultLoader::submit.type_id()], 0);
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::Submit,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultLoader::submit.type_id()], 1);

            // Open File Dialog
            assert_eq!(
                call_map.borrow()[&VaultLoader::open_file_dialog_path::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::OpenFileDialog,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::open_file_dialog_path::<TestPlatform>.type_id()],
                1
            );

            // Path selected
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 1);
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::PathSelected(Ok("path".into())),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 2);
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::PathSelected(Err(error::NfdError::Null)),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 2);

            // Open key file dialog
            assert_eq!(
                call_map.borrow()
                    [&VaultLoader::open_file_dialog_key_file::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::KeyFileOpenFD,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()
                    [&VaultLoader::open_file_dialog_key_file::<TestPlatform>.type_id()],
                1
            );

            // Key file selected
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_key_file.type_id()],
                1
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::KeyFileSelected(Ok("path".into())),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_key_file.type_id()],
                2
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::KeyFileSelected(Err(error::NfdError::Null)),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_key_file.type_id()],
                2
            );

            // Create
            let res = vault_loader
                .update::<TestPlatform>(
                    VaultLoaderMessage::Create,
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            // Loaded
            let res = vault_loader
                .update::<TestPlatform>(
                    VaultLoaderMessage::Loaded(Err(pwduck_core::PWDuckCoreError::Error("".into()))),
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
                .filter(|(k, _)| *k != &VaultLoader::update_path.type_id()
                    && *k != &VaultLoader::update_key_file.type_id())
                .all(|(_, v)| *v == 1));
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 2);
            assert_eq!(
                call_map.borrow()[&VaultLoader::update_key_file.type_id()],
                2
            );
        });
    }
}
