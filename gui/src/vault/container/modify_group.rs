//! TODO
use getset::{Getters, MutGetters, Setters};

use iced::{button, scrollable, text_input, Command, Element, Row, Scrollable, Text};
use pwduck_core::{Group, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::Icon,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space, icon_button,
        SomeIf,
    },
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING,
};

/// The state of the modify group view.
#[derive(Debug, Getters, MutGetters, Setters)]
pub struct ModifyGroupView {
    /// The group was newly created or an existing group will be modified.
    state: State,

    /// The group to modify.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    group: Group,

    /// The state of the [`TextInput`](iced::TextInput) of the title.
    title_state: text_input::State,

    /// The state of the cancel [`Button`](iced::Button).
    cancel_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,

    /// The state of the [`Scrollable`](iced::Scrollable).
    scrollable_state: scrollable::State,
}

/// The message that is send by the [`ModifyGroupView`](ModifyGroupView).
#[derive(Clone, Debug)]
pub enum ModifyGroupMessage {
    /// Change the title to the new value.
    TitleInput(String),
    /// Cancel the modification of the group.
    Cancel,
    /// Submit the modification of the group.
    Submit,
}
impl SomeIf for ModifyGroupMessage {}

impl ModifyGroupView {
    /// Create a new [`ModifyGroupView`](ModifyGroupView).
    ///
    /// It expects:
    ///     - The group to modify.
    pub fn with(state: State, group: Group) -> Self {
        Self {
            state,

            group,

            title_state: text_input::State::new(),

            cancel_state: button::State::new(),
            submit_state: button::State::new(),

            scrollable_state: scrollable::State::new(),
        }
    }

    /// Update the state of the [`ModifyGroupView`](ModifyGroupView).
    pub fn update(
        &mut self,
        message: ModifyGroupMessage,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<ModifyGroupMessage>, PWDuckGuiError> {
        match message {
            ModifyGroupMessage::TitleInput(password) => {
                self.group_mut().set_title(password);
                Ok(Command::none())
            }
            _ => PWDuckGuiError::Unreachable("ModifyGroupMessage".into()).into(),
        }
    }

    /// Create the view of the [`ModifyGroupView`](ModifyGroupView).
    pub fn view(
        &mut self,
        vault: &Vault,
        selected_group_uuid: &str,
    ) -> Element<ModifyGroupMessage> {
        let name = default_text_input(
            &mut self.title_state,
            match self.state {
                State::Create => "Enter the name of the new Group",
                State::Modify => "Enter the name of the Group",
            },
            self.group.title(),
            ModifyGroupMessage::TitleInput,
        );

        let group = vault.groups().get(selected_group_uuid).unwrap();

        let cancel = icon_button(
            &mut self.cancel_state,
            Icon::XSquare,
            "Cancel",
            "Cancel changes",
            false,
            Some(ModifyGroupMessage::Cancel),
        );

        let submit = icon_button(
            &mut self.submit_state,
            Icon::Save,
            "Submit",
            "Submit changes",
            false,
            //Some(ModifyGroupMessage::Submit),
            ModifyGroupMessage::Submit.some_if(self.group.is_modified()),
        );

        let parent_name = if group.title().is_empty() {
            "Root"
        } else {
            group.title()
        };

        let scrollable = Scrollable::new(&mut self.scrollable_state)
            .padding(DEFAULT_COLUMN_PADDING)
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(Text::new(match self.state {
                State::Create => format!("Add new sub group to: {}", parent_name),
                State::Modify => "Edit group:".into(),
            }))
            .push(name)
            .push(default_vertical_space())
            .push(
                Row::new()
                    .spacing(DEFAULT_ROW_SPACING)
                    .push(cancel)
                    .push(submit),
            );

        centered_container_with_column(vec![scrollable.into()]).into()
    }
}

/// The state of the group.
#[derive(Clone, Copy, Debug)]
pub enum State {
    /// The group was created.
    Create,
    /// An existing group will be modified.
    Modify,
}
