//! TODO
use getset::{Getters, MutGetters, Setters};

use iced::{button, text_input, Command, Element, Row, Text};
use pwduck_core::{Group, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::Icon,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space, icon_button,
    },
    DEFAULT_ROW_SPACING,
};

/// TODO
#[derive(Debug, Getters, MutGetters, Setters)]
pub struct ModifyGroupView {
    /// TODO
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
    /// TODO
    pub fn with(group: Group) -> Self {
        Self {
            group,

            group_name_state: text_input::State::new(),

            cancel_state: button::State::new(),
            submit_state: button::State::new(),
        }
    }

    /// TODO
    pub fn update(
        &mut self,
        message: ModifyGroupMessage,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<ModifyGroupMessage>, PWDuckGuiError> {
        match message {
            ModifyGroupMessage::GroupNameInput(password) => {
                self.group_mut().set_title(password);
                Ok(Command::none())
            }
            _ => PWDuckGuiError::Unreachable("ModifyGroupMessage".into()).into(),
        }
    }

    /// TODO
    pub fn view(
        &mut self,
        vault: &Vault,
        selected_group_uuid: &str,
    ) -> Element<ModifyGroupMessage> {
        let name = default_text_input(
            &mut self.group_name_state,
            "Enter the name of the new Group",
            self.group.title(),
            ModifyGroupMessage::GroupNameInput,
        );

        let group = vault.groups().get(selected_group_uuid).unwrap();

        let cancel = icon_button(
            &mut self.cancel_state,
            Icon::XSquare,
            "Cancel",
            "Cancel changes",
            false,
        )
        .on_press(ModifyGroupMessage::Cancel);

        let submit = icon_button(
            &mut self.submit_state,
            Icon::Save,
            "Submit",
            "Submit changes",
            false,
        )
        .on_press(ModifyGroupMessage::Submit);

        let parent_name = if group.title().is_empty() {
            "Root"
        } else {
            group.title()
        };

        centered_container_with_column(vec![
            Text::new(format!("Add new sub group to: {}", parent_name)).into(),
            name.into(),
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
