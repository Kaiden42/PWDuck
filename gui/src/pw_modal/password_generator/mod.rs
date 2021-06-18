//! TODO

use getset::{Getters, Setters};
use iced::{
    button, container, text_input, Column, Command, Container, Element, Length, Row, Text,
    TextInput,
};
use iced_aw::{Card, TabBar, TabLabel};
use pwduck_core::{PWDuckCoreError, PasswordInfo};
use zeroize::Zeroize;

use crate::{
    error::PWDuckGuiError,
    password_score::PasswordScore,
    utils::{centered_container_with_column, default_vertical_space, icon_button, vertical_space},
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};

mod passphrase_tab;
use passphrase_tab::{PassphraseTabMessage, PassphraseTabState};
mod password_tab;
use password_tab::{PasswordTabMessage, PasswordTabState};

/// TODO
#[derive(Debug, Default, Getters, Setters)]
pub struct PasswordGeneratorState {
    /// TODO
    #[getset(get = "pub")]
    password: String,
    /// TODO
    password_state: text_input::State,
    /// TODO
    password_show: bool,
    /// TODO
    password_show_state: button::State,
    /// TODO
    password_copy_state: button::State,
    /// TODO
    password_reroll_state: button::State,

    /// TODO
    password_score: Option<PasswordScore>,

    /// TODO
    active_tab: usize,
    /// TODO
    password_tab_state: PasswordTabState,
    /// TODO
    passphrase_tab_state: PassphraseTabState,

    /// TODO
    cancel_state: button::State,
    /// TODO
    submit_state: button::State,

    /// TODO
    #[getset(get = "pub", set = "pub")]
    target: Target,
}

impl PasswordGeneratorState {
    /// TODO
    fn estimate_password_strength(&mut self) -> Command<PasswordGeneratorMessage> {
        Command::perform(
            estimate_password_strength(self.password.clone().into()),
            PasswordGeneratorMessage::PasswordScore,
        )
    }

    /// TODO
    fn update_password(&mut self, password: String) -> Command<PasswordGeneratorMessage> {
        self.password.zeroize();
        self.password = password;

        self.estimate_password_strength()
    }

    /// TODO
    fn toogle_password_visibility(&mut self) -> Command<PasswordGeneratorMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// TODO
    fn copy_password(
        &mut self,
        clipboard: &mut iced::Clipboard,
    ) -> Command<PasswordGeneratorMessage> {
        clipboard.write(self.password.clone());
        Command::none()
    }

    /// TODO
    fn reroll_password(&mut self) -> Command<PasswordGeneratorMessage> {
        self.generate_and_update_password();
        Command::none()
    }

    /// TODO
    fn set_password_score(
        &mut self,
        score: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<PasswordGeneratorMessage> {
        self.password_score = Some(PasswordScore::new(score));
        Command::none()
    }

    /// TODO
    fn select_tab(&mut self, index: usize) -> Command<PasswordGeneratorMessage> {
        self.active_tab = index;
        self.reroll_password()
    }

    /// TODO
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

    /// TODO
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
}

/// TODO
#[derive(Clone, Debug)]
pub enum PasswordGeneratorMessage {
    /// TODO
    PasswordInput(String),
    /// TODO
    PasswordShow,
    /// TODO
    PasswordCopy,
    /// TODO
    PasswordReroll,

    /// TODO
    PasswordScore(Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError>),

    /// TODO
    TabSelected(usize),
    /// TODO
    PasswordTabMessage(PasswordTabMessage),
    /// TODO
    PassphraseTabMessage(PassphraseTabMessage),

    /// TODO
    Cancel,
    /// TODO
    Submit,
}

impl PasswordGeneratorState {
    /// TODO
    pub fn new() -> Self {
        Self {
            password_tab_state: PasswordTabState::new(),
            ..Self::default()
        }
    }

    /// TODO
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

    /// TODO
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

        let password_show = icon_button(
            &mut self.password_show_state,
            "I",
            if self.password_show {
                // TODO
                "H"
            } else {
                "S"
            },
        )
        .width(Length::Shrink)
        .on_press(PasswordGeneratorMessage::PasswordShow);

        let password_copy = icon_button(&mut self.password_copy_state, "I", "C")
            .width(Length::Shrink)
            .on_press(PasswordGeneratorMessage::PasswordCopy);

        let password_reroll = icon_button(&mut self.password_reroll_state, "I", "R")
            .width(Length::Shrink)
            .on_press(PasswordGeneratorMessage::PasswordReroll);

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

        buttons = buttons.push(
            icon_button(&mut self.cancel_state, "I", "Cancel")
                .on_press(PasswordGeneratorMessage::Cancel),
        );

        if self.target != Target::None {
            buttons = buttons.push(
                icon_button(&mut self.submit_state, "I", "Submit")
                    .on_press(PasswordGeneratorMessage::Submit),
            );
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

    /// TODO
    pub fn generate_and_update_password(&mut self) {
        self.password.zeroize();

        self.password = match self.active_tab {
            0 => self.password_tab_state.generate(),
            _ => self.passphrase_tab_state.generate(),
        }
    }
}

/// TODO
pub async fn estimate_password_strength(
    password: pwduck_core::SecString,
) -> Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError> {
    pwduck_core::password_entropy(&password)
}

/// TODO
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

/// TODO
#[derive(Debug, PartialEq)]
pub enum Target {
    /// TODO
    Creator,
    /// TODO
    EntryModifier,
    /// TODO
    None,
}

impl Default for Target {
    fn default() -> Self {
        Self::None
    }
}
