use iced::{
    button, text_input, Button, Column, Container, Element, HorizontalAlignment, Length, Row,
    Space, Text, TextInput,
};
use pwduck_core::Vault;

use crate::{
    DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING,
};
use getset::{Getters, Setters};

#[derive(Debug, Getters, Setters)]
pub struct CreateGroupView {
    #[getset(get = "pub", set = "pub")]
    group_name: String,
    group_name_state: text_input::State,

    cancel_state: button::State,
    submit_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum CreateGroupMessage {
    /// TODO
    GroupNameInput(String),
    /// TODO
    Cancel,
    /// TODO
    Submit,
}

impl CreateGroupView {
    pub fn new() -> Self {
        Self {
            group_name: String::new(),
            group_name_state: text_input::State::new(),

            cancel_state: button::State::new(),
            submit_state: button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        vault: &'a Vault,
        selected_group_uuid: &'a str,
    ) -> Element<'a, CreateGroupMessage> {
        let name = TextInput::new(
            &mut self.group_name_state,
            "Enter the name of the new Group",
            &self.group_name,
            CreateGroupMessage::GroupNameInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let group = vault.groups().get(selected_group_uuid).unwrap();

        let cancel = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(CreateGroupMessage::Cancel);

        let submit = Button::new(
            &mut self.submit_state,
            Text::new("Submit")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(CreateGroupMessage::Submit);

        let parent_name = if group.title().is_empty() {
            "Root"
        } else {
            group.title()
        };

        Container::new(
            Column::new()
                .max_width(DEFAULT_MAX_WIDTH)
                .push(Text::new(format!("Add new sub group to: {}", parent_name)))
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(name)
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
