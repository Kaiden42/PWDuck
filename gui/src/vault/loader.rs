//! TODO

use std::path::PathBuf;

use iced::{button, text_input, Command, Row, Text};
use iced_focus::Focus;
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

    /// The password of the vault.
    password: SecString,
    /// The state of the [`TextInput`](iced::TextInput) of the password.
    #[focus(enable)]
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

    /// Confirm the loading of the vault.
    fn confirm(&mut self) -> Command<VaultLoaderMessage> {
        if self.path.is_empty() || self.password.is_empty() {
            return Command::none();
        }

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

#[cfg_attr(test, mockable)]
impl Component for VaultLoader {
    type Message = VaultLoaderMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {
            path: String::new(),
            path_state: text_input::State::focused(),

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
                let cmd = self.update_path(path.to_str().ok_or(PWDuckGuiError::Option)?.to_owned());
                self.path_state.unfocus();
                self.password_state.focus();
                cmd
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

        let mut vault_path = default_text_input(
            &mut self.path_state,
            "Choose a Vault",
            &self.path,
            VaultLoaderMessage::PathInput,
        )
        .style(theme.text_input());
        if P::is_nfd_available() {
            vault_path = vault_path.on_submit(VaultLoaderMessage::OpenFileDialog);
        }

        let mut password = default_text_input(
            &mut self.password_state,
            "Password",
            &self.password,
            VaultLoaderMessage::PasswordInput,
        )
        .on_submit(VaultLoaderMessage::Confirm)
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
    fn confirm() {
        let mut vault_loader = VaultLoader::new(());
        assert!(vault_loader.path.is_empty());
        assert!(vault_loader.password.is_empty());

        let cmd = vault_loader.confirm();
        assert!(cmd.futures().is_empty());

        let _cmd = vault_loader.update_password("password".into());
        let cmd = vault_loader.confirm();
        assert!(cmd.futures().is_empty());

        let _cmd = vault_loader.update_path("path".into());
        let cmd = vault_loader.confirm();
        assert!(!cmd.futures().is_empty());

        // Password should be zeroized.
        let cmd = vault_loader.confirm();
        assert!(cmd.futures().is_empty());
    }

    #[test]
    fn open_file_dialog() {
        let cmd = VaultLoader::open_file_dialog::<TestPlatform>();
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
                .insert(VaultLoader::confirm.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultLoader::open_file_dialog::<TestPlatform>.type_id(), 0);

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
            VaultLoader::confirm.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::confirm.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultLoader::open_file_dialog::<TestPlatform>.mock_raw(|| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultLoader::open_file_dialog::<TestPlatform>.type_id())
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

            // Confirm
            assert_eq!(call_map.borrow()[&VaultLoader::confirm.type_id()], 0);
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::Confirm,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultLoader::confirm.type_id()], 1);

            // Open File Dialog
            assert_eq!(
                call_map.borrow()[&VaultLoader::open_file_dialog::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_loader.update::<TestPlatform>(
                VaultLoaderMessage::OpenFileDialog,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultLoader::open_file_dialog::<TestPlatform>.type_id()],
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
                .filter(|(k, _)| *k != &VaultLoader::update_path.type_id())
                .all(|(_, v)| *v == 1));
            assert_eq!(call_map.borrow()[&VaultLoader::update_path.type_id()], 2);
        });
    }
}
