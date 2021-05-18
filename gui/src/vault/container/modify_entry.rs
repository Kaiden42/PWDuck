//! TODO

use getset::{Getters, MutGetters};
use iced::{
    button, text_input, Button, Column, Container, Element, Length, Row, Space, Text, TextInput,
};
use pwduck_core::{EntryBody, EntryHead};

use crate::{
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING,
};

#[derive(Getters, MutGetters)]
pub struct ModifyEntryView {
    #[getset(get = "pub", get_mut = "pub")]
    entry_head: EntryHead,
    #[getset(get = "pub", get_mut = "pub")]
    entry_body: EntryBody,

    title_state: text_input::State,
    username_state: text_input::State,
    password_state: text_input::State,

    cancel_state: button::State,
    submit_state: button::State,
}

#[derive(Clone, Debug)]
pub enum ModifyEntryMessage {
    TitleInput(String),
    UsernameInput(String),
    PasswordInput(String),

    Cancel,
    Submit,
}

impl ModifyEntryView {
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

    pub fn view<'a>(
        &'a mut self,
        _selected_group_uuid: &'a str,
    ) -> Element<'a, ModifyEntryMessage> {
        let title = TextInput::new(
            &mut self.title_state,
            "Title of this entry",
            &self.entry_head.title(),
            ModifyEntryMessage::TitleInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let username = TextInput::new(
            &mut self.username_state,
            "Username",
            &self.entry_body.username(),
            ModifyEntryMessage::UsernameInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let password = TextInput::new(
            &mut self.password_state,
            "Password",
            &self.entry_body.password(),
            ModifyEntryMessage::PasswordInput,
        )
        .password()
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let cancel = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel")
                .horizontal_alignment(iced::HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(ModifyEntryMessage::Cancel);

        let submit = Button::new(
            &mut self.submit_state,
            Text::new("Submit")
                .horizontal_alignment(iced::HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(ModifyEntryMessage::Submit);

        Container::new(
            Column::new()
                .max_width(DEFAULT_MAX_WIDTH)
                .push(Text::new("Modify entry:"))
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(title)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(username)
                .push(password)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(cancel)
                        .push(submit),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

impl std::fmt::Debug for ModifyEntryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("No debug info available for ModifyEntryView")
    }
}
