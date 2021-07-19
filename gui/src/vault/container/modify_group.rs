//! TODO
use getset::{Getters, MutGetters, Setters};

use iced::{
    button, scrollable, text_input, Button, Column, Command, Element, Length, Row, Scrollable,
    Space, Text,
};
use iced_aw::{modal, Card};
use pwduck_core::{Group, Uuid, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::{Icon, ICON_FONT},
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space, icon_button,
        SomeIf,
    },
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING,
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

    /// TODO
    show_advanced: bool,
    /// TODO
    advanced_button_state: button::State,
    /// TODO
    advanced_state: AdvancedState,

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

    /// Toggle the visibility of the advanced area.
    ToggleAdvanced,
    /// The messages produced by the advanced area.
    Advanced(AdvancedStateMessage),

    /// TODO
    Modal(ModifyGroupModalMessage),
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

            show_advanced: false,
            advanced_button_state: button::State::new(),
            advanced_state: AdvancedState::new(),

            scrollable_state: scrollable::State::new(),
        }
    }

    /// Submit the modification of the group.
    fn submit(&self, vault: &mut Vault) -> Command<ModifyGroupMessage> {
        vault.insert_group(self.group.clone());
        Command::none()
    }

    /// Update the title and replace it with the given value.
    fn update_title(&mut self, title: String) -> Command<ModifyGroupMessage> {
        self.group.set_title(title);
        Command::none()
    }

    /// Toggle the visibility of the advanced area.
    fn toggle_advanced_visibility(&mut self) -> Command<ModifyGroupMessage> {
        self.show_advanced = !self.show_advanced;
        Command::none()
    }

    /// Update the advanced state with the given message.
    fn update_advanced(
        &mut self,
        message: &AdvancedStateMessage,
        vault: &Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyGroupMessage> {
        match message {
            AdvancedStateMessage::DeleteGroupRequest => {
                if vault.get_entries_of(self.group.uuid()).is_empty()
                    && vault.get_groups_of(self.group.uuid()).is_empty()
                {
                    *modal_state = modal::State::new(crate::ModalState::ModifyGroup(
                        ModifyGroupModal::delete_request(),
                    ));
                } else {
                    *modal_state = modal::State::new(crate::ModalState::ModifyGroup(
                        ModifyGroupModal::group_not_empty(),
                    ));
                }
                modal_state.show(true);
                Command::none()
            }
        }
    }

    /// Update the state of the modal.
    fn update_modal(
        &mut self,
        message: &ModifyGroupModalMessage,
        vault: &mut Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        selected_group_uuid: &mut Uuid,
    ) -> Result<Command<ModifyGroupMessage>, PWDuckGuiError> {
        match message {
            ModifyGroupModalMessage::Close => {
                *modal_state = modal::State::default();
                Ok(Command::none())
            }
            ModifyGroupModalMessage::SubmitDelete => {
                *modal_state = modal::State::default();
                *selected_group_uuid = self
                    .group()
                    .parent()
                    .as_ref()
                    .ok_or(PWDuckGuiError::Option)?
                    .clone();
                vault.delete_group(self.group.uuid());
                Ok(Command::none())
            }
        }
    }

    /// Update the state of the [`ModifyGroupView`](ModifyGroupView).
    pub fn update(
        &mut self,
        message: ModifyGroupMessage,
        vault: &mut Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        selected_group_uuid: &mut Uuid,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<ModifyGroupMessage>, PWDuckGuiError> {
        match message {
            ModifyGroupMessage::Cancel => Ok(Command::none()),
            ModifyGroupMessage::Submit => Ok(self.submit(vault)),
            ModifyGroupMessage::TitleInput(title) => Ok(self.update_title(title)),
            ModifyGroupMessage::ToggleAdvanced => Ok(self.toggle_advanced_visibility()),
            ModifyGroupMessage::Advanced(message) => {
                Ok(self.update_advanced(&message, vault, modal_state))
            }
            ModifyGroupMessage::Modal(message) => {
                self.update_modal(&message, vault, modal_state, selected_group_uuid)
            } //_ => PWDuckGuiError::Unreachable("ModifyGroupMessage".into()).into(),
        }
    }

    /// Create the view of the [`ModifyGroupView`](ModifyGroupView).
    pub fn view(
        &mut self,
        vault: &Vault,
        selected_group_uuid: &Uuid,
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

        let advanced = advanced_area(
            &mut self.advanced_button_state,
            self.show_advanced,
            &mut self.advanced_state,
            &self.group,
        );

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
            )
            .push(default_vertical_space())
            .push(advanced);

        centered_container_with_column(vec![scrollable.into()]).into()
    }
}

/// Create the advanced area.
fn advanced_area<'a>(
    button_state: &'a mut button::State,
    show_advanced: bool,
    advanced_state: &'a mut AdvancedState,
    group: &Group,
) -> Element<'a, ModifyGroupMessage> {
    let advanced_button = Button::new(
        button_state,
        Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .push(
                Text::new(if show_advanced {
                    Icon::CaretDown
                } else {
                    Icon::CaretRight
                })
                .font(ICON_FONT),
            )
            .push(Text::new("Advanced")),
    )
    .style(ToggleAdvancedButtonStyle)
    .on_press(ModifyGroupMessage::ToggleAdvanced);

    let advanced: Element<_> = if show_advanced {
        advanced_state.view(group).map(ModifyGroupMessage::Advanced)
    } else {
        Space::new(Length::Fill, Length::Shrink).into()
    };

    Column::new()
        .spacing(DEFAULT_COLUMN_SPACING)
        .push(advanced_button)
        .push(advanced)
        .into()
}

/// The state of the group.
#[derive(Clone, Copy, Debug)]
pub enum State {
    /// The group was created.
    Create,
    /// An existing group will be modified.
    Modify,
}

/// The style of the toggler to toggle the advanced view.
#[derive(Clone, Copy, Debug)]
struct ToggleAdvancedButtonStyle;

impl button::StyleSheet for ToggleAdvancedButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::new(0.0, 0.0),
            background: iced::Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::BLACK,
        }
    }
}

/// The state of the advanced view.
#[derive(Debug)]
pub struct AdvancedState {
    /// The state of the [`Button`](iced::Button) to delete the group.
    delete: button::State,
}

/// The message produced by the advanced view.
#[derive(Clone, Debug)]
pub enum AdvancedStateMessage {
    /// The deletion of a group was requested.
    DeleteGroupRequest,
}

impl AdvancedState {
    /// Create a new advanced state.
    pub fn new() -> Self {
        Self {
            delete: button::State::new(),
        }
    }

    /// Create the advanced view.
    pub fn view(&mut self, _group: &Group) -> Element<AdvancedStateMessage> {
        let delete = icon_button(
            &mut self.delete,
            Icon::Trash,
            "Delete",
            "Delete this group",
            false,
            Some(AdvancedStateMessage::DeleteGroupRequest),
        );

        Column::new()
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(default_vertical_space())
            .push(delete)
            .into()
    }
}

/// The state of the modal.
#[derive(Debug)]
pub enum ModifyGroupModal {
    /// Confirm the deletion of the group.
    DeleteRequest {
        /// The state of the cancel [`Button`](iced::Button).
        cancel_button_state: button::State,
        /// The state of the submit [`Button`](iced::Button).
        submit_button_state: button::State,
    },
    /// The group is not empty and cannot be deleted.
    GroupNotEmpty {
        /// The state of the cancel [`Button`](iced::Button).
        cancel_button_state: button::State,
    },
    /// No modal.
    None,
}

/// The message send by the modal.
#[derive(Clone, Debug)]
pub enum ModifyGroupModalMessage {
    /// Close the modal.
    Close,
    /// Submit the deletion of the group.
    SubmitDelete,
}

impl ModifyGroupModal {
    /// Create the modal to confirm the group deletion.    
    fn delete_request() -> Self {
        Self::DeleteRequest {
            cancel_button_state: button::State::new(),
            submit_button_state: button::State::new(),
        }
    }

    /// Create the modal to mention, that the group is not empty.
    fn group_not_empty() -> Self {
        Self::GroupNotEmpty {
            cancel_button_state: button::State::new(),
        }
    }

    /// Create the view of the modal.
    pub fn view(&mut self) -> Element<'_, ModifyGroupModalMessage> {
        match self {
            ModifyGroupModal::DeleteRequest { cancel_button_state, submit_button_state } => {
                Card::new(
                    Text::new("Confirm deletion"),
                    Text::new("Do you really want to delete this group? This cannot be undone!"),
                )
                .foot(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(icon_button(
                            cancel_button_state,
                            Icon::XSquare,
                            "Cancel",
                            "Cancel the deletion of the group",
                            false,
                            Some(ModifyGroupModalMessage::Close),
                        ))
                        .push(icon_button(
                            submit_button_state,
                            Icon::Save,
                            "Submit",
                            "Submit the deletion of the group",
                            false,
                            Some(ModifyGroupModalMessage::SubmitDelete),
                        )),
                )
                .max_width(DEFAULT_MAX_WIDTH)
                .into()
            },
            ModifyGroupModal::GroupNotEmpty{ cancel_button_state } => {
                Card::new(
                    Text::new("Cannot delete group"),
                    Text::new("This group is not empty and cannot be deleted. Remove all subgroups and entries first before you delete this group.")
                )
                .foot(icon_button(
                    cancel_button_state,
                    Icon::XSquare,
                    "Cancel",
                    "Close this modal",
                    false,
                    Some(ModifyGroupModalMessage::Close)
                ))
                .max_width(DEFAULT_MAX_WIDTH)
                .into()
            },
            ModifyGroupModal::None => Text::new("This message should never appear!").into(),
        }
    }
}

impl Default for ModifyGroupModal {
    fn default() -> Self {
        Self::None
    }
}
