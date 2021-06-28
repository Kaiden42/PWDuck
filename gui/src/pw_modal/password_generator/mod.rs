//! TODO

use getset::{Getters, Setters};
use iced::{
    button, container, text_input, Column, Command, Container, Element, Length, Row, Text,
    TextInput,
};
use iced_aw::{Card, TabBar, TabLabel};
use lazy_static::__Deref;
use pwduck_core::{PWDuckCoreError, PasswordInfo, SecString};

use crate::{
    error::PWDuckGuiError,
    icons::Icon,
    password_score::PasswordScore,
    utils::{
        centered_container_with_column, default_vertical_space, estimate_password_strength,
        icon_button, password_toggle, vertical_space,
    },
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};

mod passphrase_tab;
use passphrase_tab::{PassphraseTabMessage, PassphraseTabState};
mod password_tab;
use password_tab::{PasswordTabMessage, PasswordTabState};

/// The state of the password generator.
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

impl PasswordGeneratorState {
    /// Create a new [`PasswordGeneratorState`](PasswordGeneratorState).
    pub fn new() -> Self {
        Self {
            password_tab_state: PasswordTabState::new(),
            ..Self::default()
        }
    }

    /// Calculate the strength of the generated password.
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
    fn toogle_password_visibility(&mut self) -> Command<PasswordGeneratorMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// Copy the password to clipboard.
    fn copy_password(&self, clipboard: &mut iced::Clipboard) -> Command<PasswordGeneratorMessage> {
        clipboard.write(self.password.deref().clone());
        Command::none()
    }

    /// Reroll the password.
    fn reroll_password(&mut self) -> Command<PasswordGeneratorMessage> {
        self.generate_and_update_password();
        Command::none()
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
    ) -> Result<Command<PasswordGeneratorMessage>, PWDuckGuiError> {
        self.password_tab_state
            .update(message)
            .map(|cmd| cmd.map(PasswordGeneratorMessage::PasswordTabMessage))?;
        self.generate_and_update_password();

        Ok(self.estimate_password_strength())
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
            PasswordGeneratorMessage::PasswordShow => Ok(self.toogle_password_visibility()),
            PasswordGeneratorMessage::PasswordCopy => Ok(self.copy_password(clipboard)),
            PasswordGeneratorMessage::PasswordReroll => Ok(self.reroll_password()),
            PasswordGeneratorMessage::PasswordScore(score) => Ok(self.set_password_score(score)),
            PasswordGeneratorMessage::TabSelected(index) => Ok(self.select_tab(index)),
            PasswordGeneratorMessage::PasswordTabMessage(message) => {
                self.update_password_tab(&message)
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
    pub fn view(&mut self) -> Element<PasswordGeneratorMessage> {
        let head = Text::new("Generate new password");

        let mut password = TextInput::new(
            &mut self.password_state,
            "Generated password",
            &self.password,
            PasswordGeneratorMessage::PasswordInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);
        if !self.password_show {
            password = password.password();
        }

        let password_show = password_toggle(
            &mut self.password_show_state,
            self.password_show,
            PasswordGeneratorMessage::PasswordShow,
        );

        let password_copy = icon_button(
            &mut self.password_copy_state,
            Icon::FileEarmarkLock,
            "Copy Password",
            "Copy Password to clipboard",
            true,
            Some(PasswordGeneratorMessage::PasswordCopy),
        );

        let password_reroll = icon_button(
            &mut self.password_reroll_state,
            Icon::ArrowClockwise,
            "Reroll",
            "Reroll Password",
            true,
            Some(PasswordGeneratorMessage::PasswordReroll),
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
            .push(TabLabel::Text("Password".into()))
            .push(TabLabel::Text("Passphrase".into()));

        let tab_content = centered_container_with_column(vec![match self.active_tab {
            0 => self
                .password_tab_state
                .view()
                .map(PasswordGeneratorMessage::PasswordTabMessage),
            _ => self
                .passphrase_tab_state
                .view()
                .map(PasswordGeneratorMessage::PassphraseTabMessage),
        }])
        .style(TabContainerStyle)
        .height(Length::Shrink);

        let tabs = Column::new().push(tab_bar).push(tab_content);

        let mut buttons = Row::new().spacing(DEFAULT_ROW_SPACING);

        buttons = buttons.push(icon_button(
            &mut self.cancel_state,
            Icon::XSquare,
            "Cancel",
            "Cancel password generation",
            false,
            Some(PasswordGeneratorMessage::Cancel),
        ));

        if self.target != Target::None {
            buttons = buttons.push(icon_button(
                &mut self.submit_state,
                Icon::Save,
                "Submit",
                "Submit generated password",
                false,
                Some(PasswordGeneratorMessage::Submit),
            ));
        }

        let body = centered_container_with_column(vec![
            password_row.into(),
            password_score,
            vertical_space(3).into(),
            tabs.into(),
            buttons.into(),
        ])
        .height(Length::Shrink);

        Card::new(head, body)
            .max_width(DEFAULT_MAX_WIDTH)
            .on_close(PasswordGeneratorMessage::Cancel)
            .into()
    }

    /// Generate a new random password.
    pub fn generate_and_update_password(&mut self) {
        self.password = match self.active_tab {
            0 => self.password_tab_state.generate(),
            _ => self.passphrase_tab_state.generate(),
        }
        .into()
    }
}

/// The style of the tab container.
#[derive(Debug, Default)]
struct TabContainerStyle;

impl container::StyleSheet for TabContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: None,
            border_radius: 1.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.8, 0.8, 0.8),
        }
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
