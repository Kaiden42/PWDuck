//! TODO

use getset::{Getters, MutGetters, Setters};
use iced::{button, text_input, Command, Container, Element, Length, Row, Text};
use pwduck_core::{EntryBody, EntryHead, PWDuckCoreError, PasswordInfo};

use crate::{
    error::PWDuckGuiError,
    password_score::PasswordScore,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space,
        estimate_password_strength, icon_button,
    },
    DEFAULT_ROW_SPACING,
};

/// TODO
#[derive(Getters, MutGetters, Setters)]
pub struct ModifyEntryView {
    /// TODO
    #[getset(get = "pub", get_mut = "pub")]
    entry_head: EntryHead,
    /// TODO
    #[getset(get = "pub", get_mut = "pub")]
    entry_body: EntryBody,

    /// TODO
    title_state: text_input::State,
    /// TODO
    username_state: text_input::State,
    /// TODO
    username_copy_state: button::State,
    /// TODO
    password_state: text_input::State,
    /// TODO
    #[getset(get = "pub", set = "pub")]
    password_show: bool,
    /// TODO
    password_show_state: button::State,
    /// TODO
    password_generate_state: button::State,
    /// TODO
    password_copy_state: button::State,

    /// TODO
    password_score: Option<PasswordScore>,

    /// TODO
    cancel_state: button::State,
    /// TODO
    submit_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum ModifyEntryMessage {
    /// TODO
    TitleInput(String),
    /// TODO
    UsernameInput(String),
    /// TODO
    UsernameCopy,
    /// TODO
    PasswordInput(String),
    /// TODO
    PasswordShow,
    /// TODO
    PasswordGenerate,
    /// TODO
    PasswordCopy,

    /// TODO
    PasswordScore(Result<PasswordInfo, PWDuckCoreError>),

    /// TODO
    Cancel,
    /// TODO
    Submit,
}

impl ModifyEntryView {
    /// TODO
    pub fn with(entry_head: EntryHead, entry_body: EntryBody) -> Self {
        Self {
            entry_head,
            entry_body,

            title_state: text_input::State::new(),
            username_state: text_input::State::new(),
            username_copy_state: button::State::new(),
            password_state: text_input::State::new(),
            password_show: false,
            password_show_state: button::State::new(),
            password_generate_state: button::State::new(),
            password_copy_state: button::State::new(),

            password_score: Option::None,

            cancel_state: button::State::new(),
            submit_state: button::State::new(),
        }
    }

    /// TODO
    fn update_title(&mut self, title: String) -> Command<ModifyEntryMessage> {
        self.entry_head_mut().set_title(title);
        Command::none()
    }

    /// TODO
    fn update_username(&mut self, username: String) -> Command<ModifyEntryMessage> {
        self.entry_body_mut().set_username(username);
        Command::none()
    }

    /// TODO
    fn copy_username(&self, clipboard: &mut iced::Clipboard) -> Command<ModifyEntryMessage> {
        clipboard.write(self.entry_body().username().clone());
        Command::none()
    }

    /// TODO
    fn update_password(&mut self, password: String) -> Command<ModifyEntryMessage> {
        self.entry_body_mut().set_password(password);
        self.estimate_password_strength()
    }

    /// TODO
    fn toggle_password_visibility(&mut self) -> Command<ModifyEntryMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// TODO
    fn copy_password(&self, clipboard: &mut iced::Clipboard) -> Command<ModifyEntryMessage> {
        clipboard.write(self.entry_body().password().clone());
        Command::none()
    }

    /// TODO
    fn estimate_password_strength(&self) -> Command<ModifyEntryMessage> {
        Command::perform(
            estimate_password_strength(self.entry_body.password().clone().into()),
            ModifyEntryMessage::PasswordScore,
        )
    }

    /// TODO
    fn set_password_score(
        &mut self,
        password_info: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<ModifyEntryMessage> {
        self.password_score = Some(PasswordScore::new(password_info));
        Command::none()
    }

    /// TODO
    pub fn update(
        &mut self,
        message: ModifyEntryMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<ModifyEntryMessage>, PWDuckGuiError> {
        match message {
            ModifyEntryMessage::TitleInput(title) => Ok(self.update_title(title)),
            ModifyEntryMessage::UsernameInput(username) => Ok(self.update_username(username)),
            ModifyEntryMessage::UsernameCopy => Ok(self.copy_username(clipboard)),
            ModifyEntryMessage::PasswordInput(password) => Ok(self.update_password(password)),
            ModifyEntryMessage::PasswordShow => Ok(self.toggle_password_visibility()),
            ModifyEntryMessage::PasswordCopy => Ok(self.copy_password(clipboard)),
            ModifyEntryMessage::PasswordScore(password_info) => {
                Ok(self.set_password_score(password_info))
            }
            ModifyEntryMessage::PasswordGenerate
            | ModifyEntryMessage::Cancel
            | ModifyEntryMessage::Submit => {
                PWDuckGuiError::Unreachable("ModifyEntryMessage".into()).into()
            }
        }
    }

    /// TODO
    pub fn view(&mut self, _selected_group_uuid: &str) -> Element<ModifyEntryMessage> {
        let title = default_text_input(
            &mut self.title_state,
            "Title of this entry",
            self.entry_head.title(),
            ModifyEntryMessage::TitleInput,
        );

        let username = default_text_input(
            &mut self.username_state,
            "Username",
            self.entry_body.username(),
            ModifyEntryMessage::UsernameInput,
        );
        let username_copy = icon_button(&mut self.username_copy_state, "I", "C")
            .width(Length::Shrink)
            .on_press(ModifyEntryMessage::UsernameCopy);

        let mut password = default_text_input(
            &mut self.password_state,
            "Password",
            self.entry_body.password(),
            ModifyEntryMessage::PasswordInput,
        );
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
        .on_press(ModifyEntryMessage::PasswordShow);
        let password_generate = icon_button(&mut self.password_generate_state, "I", "G")
            .width(Length::Shrink)
            .on_press(ModifyEntryMessage::PasswordGenerate);
        let password_copy = icon_button(&mut self.password_copy_state, "I", "C")
            .width(Length::Shrink)
            .on_press(ModifyEntryMessage::PasswordCopy);

        let password_score: Element<_> = self.password_score.as_mut().map_or_else(
            || Container::new(default_vertical_space()).into(),
            PasswordScore::view,
        );

        let cancel =
            icon_button(&mut self.cancel_state, "I", "Cancel").on_press(ModifyEntryMessage::Cancel);

        let submit =
            icon_button(&mut self.submit_state, "I", "Submit").on_press(ModifyEntryMessage::Submit);

        centered_container_with_column(vec![
            Text::new("Modify entry:").into(),
            //default_vertical_space().into(),
            title.into(),
            default_vertical_space().into(),
            //username.into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(username)
                .push(username_copy)
                .into(),
            //password.into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(password)
                .push(password_show)
                .push(password_generate)
                .push(password_copy)
                .into(),
            password_score,
            default_vertical_space().into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(cancel)
                .push(submit)
                .into(),
        ])
        .into()
    }
}

impl std::fmt::Debug for ModifyEntryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("No debug info available for ModifyEntryView")
    }
}
