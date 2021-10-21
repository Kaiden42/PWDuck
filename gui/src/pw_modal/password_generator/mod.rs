//! The state of the password / passphrase generator.
//!
//! The passphrase generator is currently not implemented.
use getset::{Getters, Setters};
use iced::{button, text_input, Column, Command, Container, Element, Length, Row, Text, TextInput};
use iced_aw::{Card, TabBar, TabLabel};
use lazy_static::__Deref;
use pwduck_core::{PWDuckCoreError, PasswordInfo, SecString};

use crate::{
    error::PWDuckGuiError,
    icons::Icon,
    password_score::PasswordScore,
    theme::Theme,
    utils::{
        centered_container_with_column, default_vertical_space, estimate_password_strength,
        icon_button, password_toggle, vertical_space, ButtonData, ButtonKind,
    },
    vault::{
        container::ModifyEntryMessage,
        creator::VaultCreatorMessage,
        tab::{VaultContainerMessage, VaultTabMessage},
    },
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};

mod passphrase_tab;
use passphrase_tab::{PassphraseTabMessage, PassphraseTabState};
mod password_tab;
use password_tab::{PasswordTabMessage, PasswordTabState};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the password / passphrase generator.
#[derive(Debug, Default, Getters, Setters)]
pub struct PasswordGeneratorState {
    /// The generated password.
    #[getset(get = "pub")]
    password: SecString,
    /// The state of the password [`TextInput`](TextInput).
    password_state: text_input::State,
    /// The visibility of the password.
    password_show: bool,
    /// The state of the password toggle [`Button`](iced::Button).
    password_show_state: button::State,
    /// The state of the password copy [`Button`](iced::Button).
    password_copy_state: button::State,
    /// The state of the password reroll [`Button`](iced::Button).
    password_reroll_state: button::State,

    /// The estimated password score.
    password_score: Option<PasswordScore>,

    /// The index of the active tab.
    active_tab: usize,
    /// The state of the password generator tab.
    password_tab_state: PasswordTabState,
    /// The state of the passphrase generator tab.
    passphrase_tab_state: PassphraseTabState,

    /// The state of the cancel [`Button`](iced::Button).
    cancel_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,

    /// The target to generate the password for.
    #[getset(get = "pub", set = "pub")]
    target: Target,
}

/// The message that is send by the password generator.
#[derive(Clone, Debug)]
pub enum PasswordGeneratorMessage {
    /// The [`TextInput`](TextInput) for the password changed the value.
    PasswordInput(String),
    /// Toggle the password visibility.
    PasswordShow,
    /// Copy the password.
    PasswordCopy,
    /// Reroll the password.
    PasswordReroll,

    /// The estimated password score.
    PasswordScore(Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError>),

    /// A tab was selected.
    TabSelected(usize),
    /// A message produced by the password tab.
    PasswordTabMessage(PasswordTabMessage),
    /// A message produced by the passphrase tab.
    PassphraseTabMessage(PassphraseTabMessage),

    /// The cancel [`Button`](iced::Button) was pressed.
    Cancel,
    /// The submit [`Button`](iced::Button) was pressed.
    Submit,
}

#[cfg_attr(test, mockable)]
impl PasswordGeneratorState {
    /// Create a new [`PasswordGeneratorState`](PasswordGeneratorState).
    pub fn new() -> Self {
        Self {
            password_tab_state: PasswordTabState::new(),
            ..Self::default()
        }
    }

    /// Show the password modal.
    pub fn show(
        message: &crate::Message,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> iced::Command<PasswordGeneratorMessage> {
        let mut password_generator_state = Self::new();
        password_generator_state.set_target(match message {
            crate::Message::VaultTab(
                _,
                VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(_)),
            ) => Target::EntryModifier,
            crate::Message::VaultTab(_, VaultTabMessage::Creator(_)) => todo!(),
            _ => Target::None,
        });

        password_generator_state.generate_and_update_password();
        let generated_password = password_generator_state.password().clone();

        *modal_state = iced_aw::modal::State::new(crate::ModalState::Password(Box::new(
            password_generator_state,
        )));
        modal_state.show(true);

        Command::perform(
            estimate_password_strength(generated_password),
            PasswordGeneratorMessage::PasswordScore,
        )
    }

    /// Close the password modal.
    pub fn cancel(
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> iced::Command<PasswordGeneratorMessage> {
        *modal_state = iced_aw::modal::State::default();
        Command::none()
    }

    /// Submit the generated password.
    #[cfg_attr(coverage, no_coverage)]
    pub fn submit(
        &mut self,
        selected_tab: usize,
    ) -> Result<iced::Command<crate::Message>, PWDuckGuiError> {
        let password = self.password().clone();
        let message = match self.target() {
            Target::Creator => {
                VaultTabMessage::Creator(VaultCreatorMessage::PasswordInput(password.into()))
            }
            Target::EntryModifier => {
                VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(
                    ModifyEntryMessage::PasswordInput(password.into()),
                ))
            }
            Target::None => return PWDuckGuiError::Unreachable("Message".into()).into(),
        };

        Ok(Command::perform(
            async move { crate::Message::VaultTab(selected_tab, message) },
            |m| m,
        ))
    }

    /// Calculate the strength of the generated password.
    #[cfg_attr(coverage, no_coverage)]
    fn estimate_password_strength(&self) -> Command<PasswordGeneratorMessage> {
        Command::perform(
            estimate_password_strength(self.password.clone()),
            PasswordGeneratorMessage::PasswordScore,
        )
    }

    /// Update the password and replace it with the given value.
    fn update_password(&mut self, password: String) -> Command<PasswordGeneratorMessage> {
        self.password = password.into();
        self.estimate_password_strength()
    }

    /// Toggle the visibility of the password.
    fn toggle_password_visibility(&mut self) -> Command<PasswordGeneratorMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// Copy the password to clipboard.
    #[cfg_attr(coverage, no_coverage)]
    fn copy_password(&self, clipboard: &mut iced::Clipboard) -> Command<PasswordGeneratorMessage> {
        clipboard.write(self.password.deref().clone());
        Command::none()
    }

    /// Reroll the password.
    fn reroll_password(&mut self) -> Command<PasswordGeneratorMessage> {
        self.generate_and_update_password();
        self.estimate_password_strength()
    }

    /// Set the estimated score of the password.
    fn set_password_score(
        &mut self,
        score: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<PasswordGeneratorMessage> {
        self.password_score = Some(PasswordScore::new(score));
        Command::none()
    }

    /// Select the tab identified by the given index.
    fn select_tab(&mut self, index: usize) -> Command<PasswordGeneratorMessage> {
        self.active_tab = index;
        self.reroll_password()
    }

    /// Update the password tab with the given message.
    fn update_password_tab(
        &mut self,
        message: &PasswordTabMessage,
    ) -> Command<PasswordGeneratorMessage> {
        self.password_tab_state
            .update(message)
            .map(PasswordGeneratorMessage::PasswordTabMessage);
        self.generate_and_update_password();

        self.estimate_password_strength()
    }

    /// Update the passphrase tab with the given message.
    fn update_passphrase_tab(
        &mut self,
        message: &PassphraseTabMessage,
    ) -> Result<Command<PasswordGeneratorMessage>, PWDuckGuiError> {
        self.passphrase_tab_state
            .update(message)
            .map(|cmd| cmd.map(PasswordGeneratorMessage::PassphraseTabMessage))?;
        self.generate_and_update_password();

        Ok(self.estimate_password_strength())
    }

    /// Update the [`PasswordGeneratorState`](PasswordGeneratorState) with the given message.
    pub fn update(
        &mut self,
        message: PasswordGeneratorMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<PasswordGeneratorMessage>, PWDuckGuiError> {
        match message {
            PasswordGeneratorMessage::PasswordInput(password) => Ok(self.update_password(password)),
            PasswordGeneratorMessage::PasswordShow => Ok(self.toggle_password_visibility()),
            PasswordGeneratorMessage::PasswordCopy => Ok(self.copy_password(clipboard)),
            PasswordGeneratorMessage::PasswordReroll => Ok(self.reroll_password()),
            PasswordGeneratorMessage::PasswordScore(score) => Ok(self.set_password_score(score)),
            PasswordGeneratorMessage::TabSelected(index) => Ok(self.select_tab(index)),
            PasswordGeneratorMessage::PasswordTabMessage(message) => {
                Ok(self.update_password_tab(&message))
            }
            PasswordGeneratorMessage::PassphraseTabMessage(message) => {
                self.update_passphrase_tab(&message)
            }
            PasswordGeneratorMessage::Cancel | PasswordGeneratorMessage::Submit => {
                PWDuckGuiError::Unreachable("PasswordGeneratorMessage".into()).into()
            }
        }
    }

    /// Create the view of the [`PasswordGeneratorState`](PasswordGeneratorState).
    #[cfg_attr(coverage, no_coverage)]
    #[allow(clippy::too_many_lines)]
    pub fn view(&mut self, theme: &dyn Theme) -> Element<PasswordGeneratorMessage> {
        let head = Text::new("Generate new password");

        let mut password = TextInput::new(
            &mut self.password_state,
            "Generated password",
            &self.password,
            PasswordGeneratorMessage::PasswordInput,
        )
        .style(theme.text_input())
        .padding(DEFAULT_TEXT_INPUT_PADDING);
        if !self.password_show {
            password = password.password();
        }

        let password_show = password_toggle(
            &mut self.password_show_state,
            self.password_show,
            PasswordGeneratorMessage::PasswordShow,
            theme,
        );

        let password_copy = icon_button(
            ButtonData {
                state: &mut self.password_copy_state,
                icon: Icon::FileEarmarkLock,
                text: "Copy password",
                kind: ButtonKind::Normal,
                on_press: Some(PasswordGeneratorMessage::PasswordCopy),
            },
            "Copy password to clipboard",
            true,
            theme,
        );

        let password_reroll = icon_button(
            ButtonData {
                state: &mut self.password_reroll_state,
                icon: Icon::ArrowClockwise,
                text: "Reroll",
                kind: ButtonKind::Normal,
                on_press: Some(PasswordGeneratorMessage::PasswordReroll),
            },
            "Reroll password",
            true,
            theme,
        );

        let password_row = Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .push(password)
            .push(password_show)
            .push(password_copy)
            .push(password_reroll);

        let password_score: Element<_> = self.password_score.as_mut().map_or_else(
            || Container::new(default_vertical_space()).into(),
            PasswordScore::view,
        );

        let tab_bar = TabBar::new(self.active_tab, PasswordGeneratorMessage::TabSelected)
            .style(theme.tab_bar())
            .push(TabLabel::Text("Password".into()))
            .push(TabLabel::Text("Passphrase".into()));

        let tab_content = centered_container_with_column(
            vec![match self.active_tab {
                0 => self
                    .password_tab_state
                    .view(theme)
                    .map(PasswordGeneratorMessage::PasswordTabMessage),
                _ => self
                    .passphrase_tab_state
                    .view()
                    .map(PasswordGeneratorMessage::PassphraseTabMessage),
            }],
            theme,
        )
        .style(theme.container())
        .height(Length::Shrink);

        let tabs = Column::new().push(tab_bar).push(tab_content);

        let mut buttons = Row::new().spacing(DEFAULT_ROW_SPACING);

        buttons = buttons.push(icon_button(
            ButtonData {
                state: &mut self.cancel_state,
                icon: Icon::XSquare,
                text: "Cancel",
                kind: ButtonKind::Normal,
                on_press: Some(PasswordGeneratorMessage::Cancel),
            },
            "Cancel password generation",
            false,
            theme,
        ));

        if self.target != Target::None {
            buttons = buttons.push(icon_button(
                ButtonData {
                    state: &mut self.submit_state,
                    icon: Icon::Save,
                    text: "Submit",
                    kind: ButtonKind::Primary,
                    on_press: Some(PasswordGeneratorMessage::Submit),
                },
                "Submit generated password",
                false,
                theme,
            ));
        }

        let body = centered_container_with_column(
            vec![
                password_row.into(),
                password_score,
                vertical_space(3).into(),
                tabs.into(),
                buttons.into(),
            ],
            theme,
        )
        .height(Length::Shrink);

        Card::new(head, body)
            .max_width(DEFAULT_MAX_WIDTH)
            .style(theme.card())
            .on_close(PasswordGeneratorMessage::Cancel)
            .into()
    }

    /// Generate a new random password.
    pub fn generate_and_update_password(&mut self) {
        self.password = match self.active_tab {
            0 => self.password_tab_state.generate(),
            _ => self.passphrase_tab_state.generate(),
        }
        .into();
    }
}

/// The target to generate the password for.
#[derive(Debug, PartialEq)]
pub enum Target {
    /// Crate a new password for the [`VaultCreator`](crate::vault::creator::VaultCreator)
    Creator,
    /// Create a new password for the [`ModifyEntryView`](crate::vault::container::ModifyEntryView)
    EntryModifier,
    /// No target specified.
    None,
}

impl Default for Target {
    fn default() -> Self {
        Self::None
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

    use crate::error::PWDuckGuiError;

    use super::{
        passphrase_tab::{PassphraseTabMessage, PassphraseTabState},
        password_tab::{PasswordTabMessage, PasswordTabState},
        PasswordGeneratorMessage, PasswordGeneratorState, Target,
    };

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    #[test]
    fn new() {
        let state = PasswordGeneratorState::new();

        assert!(state.password.is_empty());
        assert!(!state.password_state.is_focused());
        assert!(!state.password_show);
        assert!(state.password_score.is_none());
        assert_eq!(state.active_tab, 0);
        assert_eq!(state.target, Target::default());
    }

    #[test]
    fn show() {
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);

        // Target entry modifier
        let _ = PasswordGeneratorState::show(
            &crate::Message::VaultTab(
                0,
                crate::vault::tab::VaultTabMessage::Container(
                    crate::vault::container::VaultContainerMessage::ModifyEntry(
                        crate::vault::container::ModifyEntryMessage::PasswordGenerate,
                    ),
                ),
            ),
            &mut modal_state,
        );

        if let crate::ModalState::Password(password_generator) = modal_state.inner() {
            assert!(!password_generator.password.is_empty());
            assert_eq!(password_generator.target, Target::EntryModifier);
        } else {
            panic!("Modal state should be a password generator");
        }
    }

    #[test]
    fn cancel() {
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::Password(Box::new(
            PasswordGeneratorState::new(),
        )));

        PasswordGeneratorState::cancel(&mut modal_state);

        if let crate::ModalState::None = modal_state.inner() {
        } else {
            panic!("Modal state should be None");
        }
    }

    #[test]
    fn update_password() {
        let mut state = PasswordGeneratorState::new();

        assert!(state.password.is_empty());

        let _ = state.update_password("password".into());

        assert_eq!(state.password.as_str(), "password");
    }

    #[test]
    fn toggle_password_visibility() {
        let mut state = PasswordGeneratorState::new();

        assert!(!state.password_show);

        let _ = state.toggle_password_visibility();

        assert!(state.password_show);

        let _ = state.toggle_password_visibility();

        assert!(!state.password_show);
    }

    #[test]
    fn reroll_password() {
        let mut state = PasswordGeneratorState::new();

        let previous = state.password.clone();

        let _ = state.reroll_password();

        assert_ne!(state.password.as_str(), previous.as_str());

        let previous = state.password.clone();

        let _ = state.reroll_password();

        assert_ne!(state.password.as_str(), previous.as_str());
    }

    #[test]
    fn set_password_score() {
        let mut state = PasswordGeneratorState::new();

        assert!(state.password_score.is_none());

        let _ = state.set_password_score(Err(pwduck_core::PWDuckCoreError::Error("error".into())));

        assert!(state.password_score.is_some());
    }

    #[test]
    fn select_tab() {
        let mut state = PasswordGeneratorState::new();

        assert_eq!(state.active_tab, 0);
        assert!(state.password.is_empty());

        let _ = state.select_tab(0);

        // TODO
        // assert_eq!(state.active_tab, 0);
        // assert!(!state.password.is_empty());
        //
        // let _ = state.select_tab(1);
        // assert_eq!(state.active_tab, 0);
        // assert!(!state.password.is_empty());
    }

    #[test]
    fn update_password_tab() {
        let mut state = PasswordGeneratorState::new();
        assert!(state.password.is_empty());

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(PasswordTabState::update.type_id(), 0);

            PasswordTabState::update.mock_raw(|_self, _message| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordTabState::update.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Update password tab state.
            assert_eq!(call_map.borrow()[&PasswordTabState::update.type_id()], 0);
            let _ = state.update_password_tab(&PasswordTabMessage::IncludeLower);
            assert_eq!(call_map.borrow()[&PasswordTabState::update.type_id()], 1);

            assert!(!state.password.is_empty());
        });
    }

    //#[test] TODO
    fn update_passphrase_tab() {
        let mut state = PasswordGeneratorState::new();
        assert!(state.password.is_empty());

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(PassphraseTabState::update.type_id(), 0);

            PassphraseTabState::update.mock_raw(|_self, _message| {
                call_map
                    .borrow_mut()
                    .get_mut(&PassphraseTabState::update.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Update passphrase tab state.
            assert_eq!(call_map.borrow()[&PassphraseTabState::update.type_id()], 0);
            let _ = state.update_passphrase_tab(&PassphraseTabMessage::Todo);
            assert_eq!(call_map.borrow()[&PassphraseTabState::update.type_id()], 1);

            assert!(!state.password.is_empty());
        })
    }

    #[test]
    fn update() {
        let mut state = PasswordGeneratorState::new();

        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::update_password.type_id(), 0);
            call_map.borrow_mut().insert(
                PasswordGeneratorState::toggle_password_visibility.type_id(),
                0,
            );
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::copy_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::reroll_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::set_password_score.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::select_tab.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::update_password_tab.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::update_passphrase_tab.type_id(), 0);

            PasswordGeneratorState::update_password.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::update_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::toggle_password_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::toggle_password_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::copy_password.mock_raw(|_self, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::copy_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::reroll_password.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::reroll_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::set_password_score.mock_raw(|_self, _score| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::set_password_score.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::select_tab.mock_raw(|_self, _index| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::select_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::update_password_tab.mock_raw(|_self, _message| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::update_password_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::update_passphrase_tab.mock_raw(|_self, _message| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::update_passphrase_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Update password
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update_password.type_id()],
                0
            );
            let _ = state.update(
                PasswordGeneratorMessage::PasswordInput("password".into()),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update_password.type_id()],
                1
            );

            // Toggle password visibility
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::toggle_password_visibility.type_id()],
                0
            );
            let _ = state.update(PasswordGeneratorMessage::PasswordShow, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::toggle_password_visibility.type_id()],
                1
            );

            // Copy password
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::copy_password.type_id()],
                0
            );
            let _ = state.update(PasswordGeneratorMessage::PasswordCopy, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::copy_password.type_id()],
                1
            );

            // Reroll password
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::reroll_password.type_id()],
                0
            );
            let _ = state.update(PasswordGeneratorMessage::PasswordReroll, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::reroll_password.type_id()],
                1
            );

            // Set password score
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::set_password_score.type_id()],
                0
            );
            let _ = state.update(
                PasswordGeneratorMessage::PasswordScore(Err(pwduck_core::PWDuckCoreError::Error(
                    "error".into(),
                ))),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::set_password_score.type_id()],
                1
            );

            // Select tab
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::select_tab.type_id()],
                0
            );
            let _ = state.update(PasswordGeneratorMessage::TabSelected(0), &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::select_tab.type_id()],
                1
            );

            // Update password tab
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update_password_tab.type_id()],
                0
            );
            let _ = state.update(
                PasswordGeneratorMessage::PasswordTabMessage(PasswordTabMessage::IncludeLower),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update_password_tab.type_id()],
                1
            );

            // Update passphrase tab
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update_passphrase_tab.type_id()],
                0
            );
            let _ = state.update(
                PasswordGeneratorMessage::PassphraseTabMessage(PassphraseTabMessage::Todo),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update_passphrase_tab.type_id()],
                1
            );

            // Cancel
            let res = state
                .update(PasswordGeneratorMessage::Cancel, &mut clipboard)
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            // Submit
            let res = state
                .update(PasswordGeneratorMessage::Submit, &mut clipboard)
                .expect_err("Should fail.");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            assert!(call_map.borrow().values().all(|v| *v == 1));
        })
    }

    #[test]
    fn generate_and_update_password() {
        let mut state = PasswordGeneratorState::new();
        assert!(state.password.is_empty());
        assert_eq!(state.active_tab, 0);

        state.generate_and_update_password();
        assert!(!state.password.is_empty());

        // TODO
        // let mut state = PasswordGeneratorState::new();
        // assert!(state.password.is_empty());
        // state.active_tab = 1;
        //
        // state.generate_and_update_password();
        // assert!(!state.password.is_empty());
    }

    #[test]
    fn default_target() {
        let target = Target::default();
        assert_eq!(target, Target::None);
    }
}
