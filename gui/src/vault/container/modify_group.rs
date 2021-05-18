//! TODO
use getset::{Getters, MutGetters, Setters};

use iced::{Button, Column, Container, Element, HorizontalAlignment, Length, Row, Space, Text, TextInput, button, text_input};
use pwduck_core::{Group, Vault};

use crate::{DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING};

#[derive(Debug, Getters, MutGetters, Setters)]
pub struct ModifyGroupView {
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    group: Group,
    
    /// TODO
    group_name_state: text_input::State,

    /// TODO
    cancel_state: button::State,
    /// TODO
    submit_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum ModifyGroupMessage {
    /// TODO
    GroupNameInput(String),
    /// TODO
    Cancel,
    /// TODO
    Submit,
}

impl ModifyGroupView {
    pub fn with(group: Group) -> Self {
        Self {
            group,

            group_name_state: text_input::State::new(),
            
            cancel_state: button::State::new(),
            submit_state: button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        vault: &Vault,
        selected_group_uuid: &str,
    ) -> Element<ModifyGroupMessage> {
        let name = TextInput::new(
            &mut self.group_name_state,
            "Enter the name of the new Group",
            &self.group.title(),
            ModifyGroupMessage::GroupNameInput,
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
        .on_press(ModifyGroupMessage::Cancel);

        let submit = Button::new(
            &mut self.submit_state,
            Text::new("Submit")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .on_press(ModifyGroupMessage::Submit);

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