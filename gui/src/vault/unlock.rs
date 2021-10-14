//! TODO

use std::path::PathBuf;

use iced::{button, text_input, Column, Command, Container, Length, Row, Space, Text};
use iced_focus::Focus;
use pwduck_core::{PWDuckCoreError, SecString, Vault};
use zeroize::Zeroize;

use crate::{
    error::PWDuckGuiError,
    icons::Icon,
    theme::Theme,
    utils::{default_text_input, icon_button, password_toggle, ButtonData, ButtonKind, SomeIf},
    Component, Viewport, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_HEADER_SIZE,
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT,
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the vault unlocker.
#[derive(Debug, Default, Focus)]
pub struct VaultUnlocker {
    /// The location of the vault to unlock.
    path: PathBuf,
    /// The password to unlock the vault.
    password: SecString,
    /// The state of the [`TextInput`](iced::TextInput) for the password.
    #[focus(enable)]
    password_state: text_input::State,
    /// The visibility of the password.
    password_show: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility.
    password_show_state: button::State,
    /// The state of the close [`Button`](iced::Button).
    close_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,
}

#[cfg_attr(test, mockable)]
impl VaultUnlocker {
    /// Update the password and replace it with the given value.
    fn update_password(&mut self, password: String) -> Command<VaultUnlockerMessage> {
        self.password = password.into();
        Command::none()
    }

    /// Toggle the visibility of the password.
    fn toggle_password_visibility(&mut self) -> Command<VaultUnlockerMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// Submit the unlocking of the vault.
    fn submit(&mut self) -> Command<VaultUnlockerMessage> {
        if self.password.is_empty() {
            return Command::none();
        }

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

/// The message that is send by the vault unlocker.
#[derive(Clone, Debug)]
pub enum VaultUnlockerMessage {
    /// Change the password to the new value.
    PasswordInput(String),
    /// Toggle the visibility of the password.
    PasswordShow,
    /// Cancel the unlocking of the vault.
    Close,
    /// Submit the unlocking of the vault.
    Submit,
    /// The vault was successfully unlocked.
    Unlocked(Result<Box<Vault>, PWDuckCoreError>),
}
impl SomeIf for VaultUnlockerMessage {}

#[cfg_attr(test, mockable)]
impl Component for VaultUnlocker {
    type Message = VaultUnlockerMessage;
    type ConstructorParam = PathBuf;

    fn new(path: Self::ConstructorParam) -> Self {
        //Self { ..Self::default() }
        Self {
            path,
            password_state: text_input::State::focused(),
            ..Self::default()
        }
    }

    fn title(&self) -> String {
        format!(
            "Unlock vault: {}",
            self.path
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("")
        )
    }

    fn update<P: crate::Platform + 'static>(
        &mut self,
        message: Self::Message,
        _application_settings: &mut pwduck_core::ApplicationSettings,
        _modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError> {
        match message {
            VaultUnlockerMessage::PasswordInput(password) => Ok(self.update_password(password)),
            VaultUnlockerMessage::PasswordShow => Ok(self.toggle_password_visibility()),
            VaultUnlockerMessage::Submit => Ok(self.submit()),
            VaultUnlockerMessage::Close | VaultUnlockerMessage::Unlocked(_) => {
                PWDuckGuiError::Unreachable("VaultUnlockerMessage".into()).into()
            }
        }
    }

    #[cfg_attr(coverage, no_coverage)]
    fn view<P: crate::Platform + 'static>(
        &mut self,
        _application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        _viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message> {
        let path = PathBuf::from(&self.path);
        let vault_name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("Name of Vault");

        let path = Text::new(self.path.to_str().unwrap_or("Invalid path"));

        let mut password = default_text_input(
            &mut self.password_state,
            "Enter password to unlock",
            &self.password,
            VaultUnlockerMessage::PasswordInput,
        )
        .on_submit(VaultUnlockerMessage::Submit)
        .style(theme.text_input());
        if !self.password_show {
            password = password.password();
        }

        let password_show = password_toggle(
            &mut self.password_show_state,
            self.password_show,
            VaultUnlockerMessage::PasswordShow,
            theme,
        );

        let close_button = icon_button(
            ButtonData {
                state: &mut self.close_state,
                icon: Icon::XSquare,
                text: "Close",
                kind: ButtonKind::Normal,
                on_press: Some(VaultUnlockerMessage::Close),
            },
            "Close vault",
            false,
            theme,
        );

        let submit_button = icon_button(
            ButtonData {
                state: &mut self.submit_state,
                icon: Icon::Unlock,
                text: "Unlock",
                kind: ButtonKind::Primary,
                on_press: VaultUnlockerMessage::Submit.some_if_not(self.password.is_empty()),
            },
            "Unlock vault",
            false,
            theme,
        );

        Container::new(
            Column::new()
                .max_width(DEFAULT_MAX_WIDTH)
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_COLUMN_SPACING)
                .push(Text::new(&format!("Unlock vault: {}", vault_name)).size(DEFAULT_HEADER_SIZE))
                .push(path)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(password)
                        .push(password_show),
                )
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(close_button)
                        .push(submit_button),
                ),
        )
        .style(theme.container())
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
        path::PathBuf,
        str::FromStr,
    };

    use iced::Command;
    use mocktopus::mocking::*;
    use pwduck_core::PWDuckCoreError;

    use crate::{error::PWDuckGuiError, Component, TestPlatform};

    use super::{VaultUnlocker, VaultUnlockerMessage};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    #[test]
    fn update_password() {
        let mut vault_unlocker = VaultUnlocker::new(".".into());
        assert!(vault_unlocker.password.is_empty());

        let _cmd = vault_unlocker.update_password("password".into());

        assert!(!vault_unlocker.password.is_empty());
        assert_eq!(vault_unlocker.password.as_str(), "password");
    }

    #[test]
    fn toggle_password_visibility() {
        let mut vault_unlocker = VaultUnlocker::new(".".into());
        assert!(!vault_unlocker.password_show);

        let _cmd = vault_unlocker.toggle_password_visibility();

        assert!(vault_unlocker.password_show);

        let _cmd = vault_unlocker.toggle_password_visibility();

        assert!(!vault_unlocker.password_show);
    }

    #[test]
    fn submit() {
        let mut vault_unlocker = VaultUnlocker::new(".".into());
        assert!(vault_unlocker.password.is_empty());

        let cmd = vault_unlocker.submit();
        assert!(cmd.futures().is_empty());

        let _cmd = vault_unlocker.update_password("password".into());

        let cmd = vault_unlocker.submit();
        assert!(!cmd.futures().is_empty());
    }

    #[test]
    fn new() {
        let vault_unlocker = VaultUnlocker::new("this/is/a/path".into());

        assert_eq!(
            &vault_unlocker.path,
            &PathBuf::from_str("this/is/a/path").unwrap()
        );
        assert!(vault_unlocker.password.is_empty());
        assert!(vault_unlocker.password_state.is_focused());
        assert!(!vault_unlocker.password_show);
    }

    #[test]
    fn title() {
        let vault_unlocker = VaultUnlocker::new("this/is/a/path".into());
        assert_eq!(vault_unlocker.title().as_str(), "Unlock vault: path");
    }

    #[test]
    fn update() {
        let mut vault_unlocker = VaultUnlocker::new(".".into());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultUnlocker::update_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultUnlocker::toggle_password_visibility.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultUnlocker::submit.type_id(), 0);

            VaultUnlocker::update_password.mock_raw(|_self, _password| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultUnlocker::update_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultUnlocker::toggle_password_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultUnlocker::toggle_password_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultUnlocker::submit.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultUnlocker::submit.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Update password
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::update_password.type_id()],
                0
            );
            let _ = vault_unlocker.update::<TestPlatform>(
                VaultUnlockerMessage::PasswordInput("password".into()),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::update_password.type_id()],
                1
            );

            // Toggle password visibility
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::toggle_password_visibility.type_id()],
                0
            );
            let _ = vault_unlocker.update::<TestPlatform>(
                VaultUnlockerMessage::PasswordShow,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultUnlocker::toggle_password_visibility.type_id()],
                1
            );

            // Submit
            assert_eq!(call_map.borrow()[&VaultUnlocker::submit.type_id()], 0);
            let _ = vault_unlocker.update::<TestPlatform>(
                VaultUnlockerMessage::Submit,
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&VaultUnlocker::submit.type_id()], 1);

            // Close
            let res = vault_unlocker
                .update::<TestPlatform>(
                    VaultUnlockerMessage::Close,
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            // Unlocked
            let res = vault_unlocker
                .update::<TestPlatform>(
                    VaultUnlockerMessage::Unlocked(Err(PWDuckCoreError::Error("".into()))),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            assert!(call_map.borrow().values().all(|v| *v == 1));
        });
    }
}
