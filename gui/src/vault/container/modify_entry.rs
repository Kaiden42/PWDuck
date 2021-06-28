//! TODO

use getset::{Getters, MutGetters, Setters};
use iced::{button, text_input, Command, Container, Element, Row, Text};
use pwduck_core::{EntryBody, EntryHead, PWDuckCoreError, PasswordInfo};

use crate::{
    error::PWDuckGuiError,
    icons::Icon,
    password_score::PasswordScore,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space,
        estimate_password_strength, icon_button, password_toggle,
    },
    DEFAULT_ROW_SPACING,
};

/// The state of the modify entry view.
#[derive(Getters, MutGetters, Setters)]
pub struct ModifyEntryView {
    /// The entry was newly created or an existing entry will be modified.
    state: State,

    /// The decrypted head of the entry to modify.
    #[getset(get = "pub", get_mut = "pub")]
    entry_head: EntryHead,
    /// The decrypted body of the entry to modify.
    #[getset(get = "pub", get_mut = "pub")]
    entry_body: EntryBody,

    /// The state of the [`TextInput`](iced::TextInput) of the title.
    title_state: text_input::State,
    /// The state of the [`TextInput`](iced::TextInput) of the usermane.
    username_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to copy the username.
    username_copy_state: button::State,
    /// The state of the [`TextInput`](iced::TextInput) of the password.
    password_state: text_input::State,
    /// The visibility of the password.
    #[getset(get = "pub", set = "pub")]
    password_show: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility of the password.
    password_show_state: button::State,
    /// The state of the [`Button`](iced::Button) to open the password generator.
    password_generate_state: button::State,
    /// The state of the [`Button`](iced::Button) to copy the password.
    password_copy_state: button::State,

    /// The estimated password score.
    password_score: Option<PasswordScore>,

    /// The state of the cancel [`Button`](iced::Button).
    cancel_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,
}

/// The message that is send by the `ModifyEntryView`.
#[derive(Clone, Debug)]
pub enum ModifyEntryMessage {
    /// Change the title to the new value.
    TitleInput(String),
    /// Change the username to the new value.
    UsernameInput(String),
    /// Copy the username.
    UsernameCopy,
    /// Change the password to the new value.
    PasswordInput(String),
    /// Toggle the visibility of the password.
    PasswordShow,
    /// Open the password generator.
    PasswordGenerate,
    /// Copy the password.
    PasswordCopy,

    /// Set the password score tho the new value.
    PasswordScore(Result<PasswordInfo, PWDuckCoreError>),

    /// Cancel the modification of the entry.
    Cancel,
    /// Submit the modification of the entry.
    Submit,
}

impl ModifyEntryView {
    /// Create a new [`ModifyEntryView`](ModifyEntryView).
    ///
    /// It expects:
    ///     - A new entry was created or an existing will be modified.
    ///     - The head of the entry to modify.
    ///     - The body of teh entry to modify.
    pub fn with(state: State, entry_head: EntryHead, entry_body: EntryBody) -> Self {
        Self {
            state,

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

    /// Update the title and replace it with the given value.
    fn update_title(&mut self, title: String) -> Command<ModifyEntryMessage> {
        self.entry_head_mut().set_title(title);
        Command::none()
    }

    /// Update the username and replace it with the given value.
    fn update_username(&mut self, username: String) -> Command<ModifyEntryMessage> {
        self.entry_body_mut().set_username(username);
        Command::none()
    }

    /// Copy the username to clipboard.
    fn copy_username(&self, clipboard: &mut iced::Clipboard) -> Command<ModifyEntryMessage> {
        clipboard.write(self.entry_body().username().clone());
        Command::none()
    }

    /// Update the password and replace it with the given value.
    fn update_password(&mut self, password: String) -> Command<ModifyEntryMessage> {
        self.entry_body_mut().set_password(password);
        self.estimate_password_strength()
    }

    /// Toggle the visibility of the password.
    fn toggle_password_visibility(&mut self) -> Command<ModifyEntryMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// Copy the password to the clipboard.
    fn copy_password(&self, clipboard: &mut iced::Clipboard) -> Command<ModifyEntryMessage> {
        clipboard.write(self.entry_body().password().clone());
        Command::none()
    }

    /// Estimate the strength of the password.
    fn estimate_password_strength(&self) -> Command<ModifyEntryMessage> {
        Command::perform(
            estimate_password_strength(self.entry_body.password().clone().into()),
            ModifyEntryMessage::PasswordScore,
        )
    }

    /// Set the estimated password score.
    fn set_password_score(
        &mut self,
        password_info: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<ModifyEntryMessage> {
        self.password_score = Some(PasswordScore::new(password_info));
        Command::none()
    }

    /// Update the state of the [`ModifyEntryView`](ModifyEntryView).
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

    /// Create the view of the [`ModifyEntryView`](ModifyEntryView).
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
        let username_copy = icon_button(
            &mut self.username_copy_state,
            Icon::FileEarmarkPerson,
            "Copy Username",
            "Copy Username to clipboard",
            true,
            Some(ModifyEntryMessage::UsernameCopy),
        );

        let mut password = default_text_input(
            &mut self.password_state,
            "Password",
            self.entry_body.password(),
            ModifyEntryMessage::PasswordInput,
        );
        if !self.password_show {
            password = password.password();
        }

        let password_show = password_toggle(
            &mut self.password_show_state,
            self.password_show,
            ModifyEntryMessage::PasswordShow,
        );

        let password_generate = icon_button(
            &mut self.password_generate_state,
            Icon::Dice3,
            "Generate Password",
            "Generate a random password",
            true,
            Some(ModifyEntryMessage::PasswordGenerate),
        );
        let password_copy = icon_button(
            &mut self.password_copy_state,
            Icon::FileEarmarkLock,
            "Copy Password",
            "Copy Password to clipboard",
            true,
            Some(ModifyEntryMessage::PasswordCopy),
        );

        let password_score: Element<_> = self.password_score.as_mut().map_or_else(
            || Container::new(default_vertical_space()).into(),
            PasswordScore::view,
        );

        let cancel = icon_button(
            &mut self.cancel_state,
            Icon::XSquare,
            "Cancel",
            "Cancel changes",
            false,
            Some(ModifyEntryMessage::Cancel),
        );

        let submit = icon_button(
            &mut self.submit_state,
            Icon::Save,
            "Submit",
            "Submit changes",
            false,
            Some(ModifyEntryMessage::Submit),
        );

        centered_container_with_column(vec![
            Text::new(match self.state {
                State::Create => "Create new entry:",
                State::Modify => "Edit entry:",
            })
            .into(),
            title.into(),
            default_vertical_space().into(),
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(username)
                .push(username_copy)
                .into(),
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

/// The state of the entry
#[derive(Clone, Copy, Debug)]
pub enum State {
    /// The entry was created.
    Create,
    /// An existing entry will be modified.
    Modify,
}
