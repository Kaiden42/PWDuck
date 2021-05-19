//! TODO

use getset::{Getters, MutGetters};
use iced::{button, text_input, Element, Row, Text};
use pwduck_core::{EntryBody, EntryHead};

use crate::{
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space, icon_button,
    },
    DEFAULT_ROW_SPACING,
};

/// TODO
#[derive(Getters, MutGetters)]
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
    password_state: text_input::State,

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
    PasswordInput(String),

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
            password_state: text_input::State::new(),

            cancel_state: button::State::new(),
            submit_state: button::State::new(),
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

        let password = default_text_input(
            &mut self.password_state,
            "Password",
            self.entry_body.password(),
            ModifyEntryMessage::PasswordInput,
        )
        .password();

        let cancel =
            icon_button(&mut self.cancel_state, "I", "Cancel").on_press(ModifyEntryMessage::Cancel);

        let submit =
            icon_button(&mut self.submit_state, "I", "Submit").on_press(ModifyEntryMessage::Submit);

        centered_container_with_column(vec![
            Text::new("Modify entry:").into(),
            //default_vertical_space().into(),
            title.into(),
            default_vertical_space().into(),
            username.into(),
            password.into(),
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
