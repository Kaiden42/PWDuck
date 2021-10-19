//! TODO
use getset::{CopyGetters, Getters, MutGetters, Setters};

use iced::{
    button, scrollable, text_input, Button, Column, Command, Element, Length, Row, Scrollable,
    Space, Text,
};
use iced_aw::{modal, Card};
use iced_focus::Focus;
use pwduck_core::{Group, Uuid, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::{Icon, ICON_FONT},
    theme::Theme,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space, icon_button,
        ButtonData, ButtonKind, SomeIf,
    },
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING,
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the modify group view.
#[derive(Debug, CopyGetters, Getters, MutGetters, Setters, Focus)]
pub struct ModifyGroupView {
    /// The group was newly created or an existing group will be modified.
    state: State,

    /// The group to modify.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    group: Group,

    /// The state of the [`TextInput`](iced::TextInput) of the title.
    #[focus(enable)]
    title_state: text_input::State,

    /// The state of the cancel [`Button`](iced::Button).
    cancel_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,

    /// Whether the group was modified or not.
    is_modified: bool,

    /// TODO
    #[getset(get_copy)]
    show_advanced: bool,
    /// TODO
    advanced_button_state: button::State,
    /// TODO
    #[focus(enable = "self.show_advanced")]
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

#[cfg_attr(test, mockable)]
impl ModifyGroupView {
    /// Create a new [`ModifyGroupView`](ModifyGroupView).
    ///
    /// It expects:
    ///     - The group to modify.
    pub fn with(state: State, group: Group) -> Self {
        Self {
            state,

            group,

            title_state: if state == State::Create {
                text_input::State::focused()
            } else {
                text_input::State::new()
            },

            is_modified: false,

            cancel_state: button::State::new(),
            submit_state: button::State::new(),

            show_advanced: false,
            advanced_button_state: button::State::new(),
            advanced_state: AdvancedState::new(),

            scrollable_state: scrollable::State::new(),
        }
    }

    /// True, if the group contains unsaved changes
    #[allow(clippy::missing_const_for_fn)]
    pub fn contains_unsaved_changes(&self) -> bool {
        self.group.is_modified()
    }

    /// Submit the modification of the group.
    fn submit(&self, vault: &mut Vault) -> Command<ModifyGroupMessage> {
        vault.insert_group(self.group.clone());
        Command::none()
    }

    /// Update the title and replace it with the given value.
    fn update_title(&mut self, title: String) -> Command<ModifyGroupMessage> {
        self.group.set_title(title);
        self.is_modified = true;
        Command::none()
    }

    /// Toggle the visibility of the advanced area.
    fn toggle_advanced_visibility(&mut self) -> Command<ModifyGroupMessage> {
        self.show_advanced = !self.show_advanced;
        Command::none()
    }

    /// Request the deletion of a group from the user.
    ///
    /// If the group is empty a confirmation modal will be shown, else an error modal.
    fn request_group_deletion(
        &mut self,
        vault: &Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyGroupMessage> {
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

    /// Update the advanced state with the given message.
    fn update_advanced(
        &mut self,
        message: &AdvancedStateMessage,
        vault: &Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyGroupMessage> {
        match message {
            AdvancedStateMessage::DeleteGroupRequest => {
                self.request_group_deletion(vault, modal_state)
            }
        }
    }

    /// Close the modal.
    #[allow(clippy::unused_self)]
    fn close_modal(
        &mut self,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyGroupMessage> {
        *modal_state = modal::State::default();
        Command::none()
    }

    /// Delete the group from the vault.
    fn delete_group(&mut self, vault: &mut Vault) -> Command<ModifyGroupMessage> {
        vault.delete_group(self.group.uuid());
        Command::none()
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
            ModifyGroupModalMessage::Close => Ok(self.close_modal(modal_state)),
            ModifyGroupModalMessage::SubmitDelete => {
                // Set current selected group uuid to parent.
                *selected_group_uuid = self
                    .group()
                    .parent()
                    .as_ref()
                    .ok_or(PWDuckGuiError::Option)?
                    .clone();

                let _cmd = self.delete_group(vault);
                Ok(self.close_modal(modal_state))
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
    #[cfg_attr(coverage, no_coverage)]
    pub fn view(
        &mut self,
        vault: &Vault,
        selected_group_uuid: &Uuid,
        theme: &dyn Theme,
    ) -> Element<ModifyGroupMessage> {
        let name = default_text_input(
            &mut self.title_state,
            match self.state {
                State::Create => "Enter the name of the new Group",
                State::Modify => "Enter the name of the Group",
            },
            self.group.title(),
            ModifyGroupMessage::TitleInput,
        )
        .style(theme.text_input());

        let group = vault.groups().get(selected_group_uuid).unwrap();

        let cancel = icon_button(
            ButtonData {
                state: &mut self.cancel_state,
                icon: Icon::XSquare,
                text: "Cancel",
                kind: ButtonKind::Normal,
                on_press: Some(ModifyGroupMessage::Cancel),
            },
            "Cancel changes",
            false,
            theme,
        );

        let submit = icon_button(
            ButtonData {
                state: &mut self.submit_state,
                icon: Icon::Save,
                text: "Submit",
                kind: ButtonKind::Primary,
                on_press: ModifyGroupMessage::Submit.some_if(self.is_modified),
            },
            "Submit changes",
            false,
            theme,
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
            self.state,
            &self.group,
            theme,
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

        centered_container_with_column(vec![scrollable.into()], theme).into()
    }
}

/// Create the advanced area.
#[cfg_attr(coverage, no_coverage)]
fn advanced_area<'a>(
    button_state: &'a mut button::State,
    show_advanced: bool,
    advanced_state: &'a mut AdvancedState,
    state: State,
    group: &Group,
    theme: &dyn Theme,
) -> Element<'a, ModifyGroupMessage> {
    if state == State::Create {
        return default_vertical_space().into();
    }

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
    .style(theme.toggle_button_advanced_area())
    .on_press(ModifyGroupMessage::ToggleAdvanced);

    let advanced: Element<_> = if show_advanced {
        advanced_state
            .view(group, theme)
            .map(ModifyGroupMessage::Advanced)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// The group was created.
    Create,
    /// An existing group will be modified.
    Modify,
}

/// The state of the advanced view.
#[derive(Debug, Focus)]
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
    #[cfg_attr(coverage, no_coverage)]
    pub fn view(&mut self, _group: &Group, theme: &dyn Theme) -> Element<AdvancedStateMessage> {
        let delete = icon_button(
            ButtonData {
                state: &mut self.delete,
                icon: Icon::Trash,
                text: "Delete",
                kind: ButtonKind::Warning,
                on_press: Some(AdvancedStateMessage::DeleteGroupRequest),
            },
            "Delete this group",
            false,
            theme,
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
    #[cfg_attr(coverage, no_coverage)]
    pub fn view(&mut self, theme: &dyn Theme) -> Element<'_, ModifyGroupModalMessage> {
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
                            ButtonData {
                                state: cancel_button_state,
                                icon: Icon::XSquare,
                                text: "Cancel",
                                kind: ButtonKind::Normal,
                                on_press: Some(ModifyGroupModalMessage::Close),
                            },
                            "Cancel the deletion of the group",
                            false,
                            theme
                        ))
                        .push(icon_button(
                            ButtonData {
                                state: submit_button_state,
                                icon: Icon::Save,
                                text: "Submit",
                                kind: ButtonKind::Warning,
                                on_press: Some(ModifyGroupModalMessage::SubmitDelete),
                            },
                            "Submit the deletion of the group",
                            false,
                            theme
                        ))
                )
                .style(theme.card_warning())
                .max_width(DEFAULT_MAX_WIDTH)
                .into()
            },
            ModifyGroupModal::GroupNotEmpty{ cancel_button_state } => {
                Card::new(
                    Text::new("Cannot delete group"),
                    Text::new("This group is not empty and cannot be deleted. Remove all subgroups and entries first before you delete this group.")
                )
                .foot(icon_button(
                    ButtonData {
                        state: cancel_button_state,
                        icon: Icon::XSquare,
                        text: "Cancel",
                        kind: ButtonKind::Normal,
                        on_press: Some(ModifyGroupModalMessage::Close),
                    },
                    "Close this modal",
                    false,
                    theme
                ))
                .style(theme.card_warning())
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

#[cfg(test)]
mod tests {

    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
    };

    use iced::Command;
    use mocktopus::mocking::*;
    use pwduck_core::{uuid, Uuid};
    use tempfile::tempdir;

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    use super::{
        AdvancedStateMessage, ModifyGroupMessage, ModifyGroupModal, ModifyGroupModalMessage,
        ModifyGroupView, State,
    };

    const DEFAULT_TITLE: &'static str = "default title";

    fn default_mgv_with_parent(parent: Uuid) -> ModifyGroupView {
        let group = pwduck_core::Group::new([1; uuid::SIZE].into(), parent, DEFAULT_TITLE.into());

        ModifyGroupView::with(State::Create, group)
    }

    fn default_mgv() -> ModifyGroupView {
        default_mgv_with_parent([0; uuid::SIZE].into())
    }

    fn equal_groups(a: &pwduck_core::Group, b: &pwduck_core::Group) -> bool {
        a.uuid() == b.uuid() && a.parent() == b.parent() && a.title() == b.title()
    }

    #[test]
    fn with() {
        let group = pwduck_core::Group::new(
            [1; uuid::SIZE].into(),
            [0; uuid::SIZE].into(),
            "title".into(),
        );

        let mgv = ModifyGroupView::with(State::Create, group.clone());
        assert_eq!(mgv.state, State::Create);
        assert!(equal_groups(&mgv.group, &group));
        assert!(mgv.title_state.is_focused());
        assert!(!mgv.is_modified);
        assert!(!mgv.show_advanced);

        let mgv = ModifyGroupView::with(State::Modify, group.clone());
        assert_eq!(mgv.state, State::Modify);
        assert!(equal_groups(&mgv.group, &group));
        assert!(!mgv.title_state.is_focused());
        assert!(!mgv.is_modified);
        assert!(!mgv.show_advanced);
    }

    #[test]
    fn contains_unsaved_changes() {
        let mgv = default_mgv();

        assert!(mgv.group.is_modified());
        assert!(mgv.contains_unsaved_changes());

        // TODO find a way to pass mocking from core to gui
    }

    #[test]
    fn submit() {
        let mgv = default_mgv();

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();
        let root = vault.get_root_uuid().unwrap();

        assert!(vault
            .get_item_list_for(&root, Some("default"))
            .groups()
            .is_empty());

        let expected_group = mgv.group().clone();

        let _ = mgv.submit(&mut vault);

        assert!(equal_groups(
            &expected_group,
            vault
                .get_item_list_for(&root, Some("default"))
                .groups()
                .first()
                .unwrap(),
        ))
    }

    #[test]
    fn update_title() {
        let mut mgv = default_mgv();

        assert_eq!(mgv.group().title().as_str(), DEFAULT_TITLE);
        assert!(!mgv.is_modified);

        let _ = mgv.update_title("title".into());

        assert_eq!(mgv.group().title().as_str(), "title");
        assert!(mgv.is_modified);
    }

    #[test]
    fn toggle_advanced_visibility() {
        let mut mgv = default_mgv();

        assert!(!mgv.show_advanced);

        let _ = mgv.toggle_advanced_visibility();

        assert!(mgv.show_advanced);

        let _ = mgv.toggle_advanced_visibility();

        assert!(!mgv.show_advanced);
    }

    #[test]
    fn request_group_deletion() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();
        let root = vault.get_root_uuid().unwrap();

        let mut mgv = default_mgv_with_parent(root.clone());
        mgv.submit(&mut vault);

        // Empty group should be deleteable
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);
        assert!(!modal_state.is_shown());
        let _ = mgv.request_group_deletion(&vault, &mut modal_state);
        if let crate::ModalState::ModifyGroup(ModifyGroupModal::DeleteRequest { .. }) =
            modal_state.inner()
        {
        } else {
            panic!("Modal should be a delete request");
        }
        assert!(modal_state.is_shown());

        // Non-empty group should not be deletable
        vault.insert_group(pwduck_core::Group::new(
            [255; uuid::SIZE].into(),
            mgv.group().uuid().clone(),
            "title".into(),
        ));
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);
        assert!(!modal_state.is_shown());
        let _ = mgv.request_group_deletion(&vault, &mut modal_state);
        if let crate::ModalState::ModifyGroup(ModifyGroupModal::GroupNotEmpty { .. }) =
            modal_state.inner()
        {
        } else {
            panic!("Modal should be a not empty warning");
        }
        assert!(modal_state.is_shown());
    }

    #[test]
    fn update_advanced() {
        let mut mgv = default_mgv();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let vault = pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
            .unwrap();

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::request_group_deletion.type_id(), 0);

            ModifyGroupView::request_group_deletion.mock_raw(|_self, _vault, _modal| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::request_group_deletion.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Request group deletion
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::request_group_deletion.type_id()],
                0
            );
            let _ = mgv.update_advanced(
                &AdvancedStateMessage::DeleteGroupRequest,
                &vault,
                &mut modal_state,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::request_group_deletion.type_id()],
                1
            );
        })
    }

    #[test]
    fn close_modal() {
        let mut mgv = default_mgv();

        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::ModifyGroup(
            ModifyGroupModal::delete_request(),
        ));

        if let crate::ModalState::ModifyGroup(ModifyGroupModal::DeleteRequest { .. }) =
            modal_state.inner()
        {
        } else {
            panic!("Modal state should be a delete request");
        }

        let _ = mgv.close_modal(&mut modal_state);

        if let crate::ModalState::None = modal_state.inner() {
        } else {
            panic!("Modal state should be None");
        }
    }

    #[test]
    fn delete_group() {
        let mut mgv = default_mgv();

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();
        let root = vault.get_root_uuid().unwrap();

        let _ = mgv.submit(&mut vault);

        let expected_group = mgv.group().clone();

        assert!(equal_groups(
            &expected_group,
            vault
                .get_item_list_for(&root, Some("default"))
                .groups()
                .first()
                .unwrap(),
        ));

        let _ = mgv.delete_group(&mut vault);

        assert!(vault
            .get_item_list_for(&root, Some("default"))
            .groups()
            .first()
            .is_none());
    }

    #[test]
    fn update_modal() {
        let mut mgv = default_mgv();

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();

        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);

        let mut selected_group_uuid: Uuid = [255; uuid::SIZE].into();

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::close_modal.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::delete_group.type_id(), 0);

            ModifyGroupView::close_modal.mock_raw(|_self, _modal| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::close_modal.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyGroupView::delete_group.mock_raw(|_self, _modal| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::delete_group.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Close modal
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::close_modal.type_id()],
                0
            );
            let _ = mgv.update_modal(
                &ModifyGroupModalMessage::Close,
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::close_modal.type_id()],
                1
            );
            assert_eq!(selected_group_uuid, Uuid::from([255; uuid::SIZE]));

            // Delete group
            let parent = mgv.group().parent().as_ref().unwrap().clone();
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::delete_group.type_id()],
                0
            );
            let _ = mgv.update_modal(
                &ModifyGroupModalMessage::SubmitDelete,
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::delete_group.type_id()],
                1
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::close_modal.type_id()],
                2
            );
            assert_eq!(selected_group_uuid, parent);
        })
    }

    #[test]
    fn update() {
        let mut mgv = default_mgv();

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();

        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);

        let mut selected_group_uuid: Uuid = [255; uuid::SIZE].into();

        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::submit.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::update_title.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::toggle_advanced_visibility.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::update_advanced.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::update_modal.type_id(), 0);

            ModifyGroupView::submit.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::submit.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyGroupView::update_title.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::update_title.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyGroupView::toggle_advanced_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::toggle_advanced_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyGroupView::update_advanced.mock_raw(|_self, _message, _vault, _state| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::update_advanced.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyGroupView::update_modal.mock_raw(|_self, _message, _vault, _state, _uuid| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::update_modal.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Cancel
            let _ = mgv
                .update(
                    ModifyGroupMessage::Cancel,
                    &mut vault,
                    &mut modal_state,
                    &mut selected_group_uuid,
                    &mut clipboard,
                )
                .expect("Should not fail");

            // Submit
            assert_eq!(call_map.borrow()[&ModifyGroupView::submit.type_id()], 0);
            let _ = mgv.update(
                ModifyGroupMessage::Submit,
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&ModifyGroupView::submit.type_id()], 1);

            // Update title
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::update_title.type_id()],
                0
            );
            let _ = mgv.update(
                ModifyGroupMessage::TitleInput("Title".into()),
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::update_title.type_id()],
                1
            );

            // Toggle advanced visibility
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::toggle_advanced_visibility.type_id()],
                0
            );
            let _ = mgv.update(
                ModifyGroupMessage::ToggleAdvanced,
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::toggle_advanced_visibility.type_id()],
                1
            );

            // Update advanced
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::update_advanced.type_id()],
                0
            );
            let _ = mgv.update(
                ModifyGroupMessage::Advanced(AdvancedStateMessage::DeleteGroupRequest),
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::update_advanced.type_id()],
                1
            );

            // Update modal
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::update_modal.type_id()],
                0
            );
            let _ = mgv.update(
                ModifyGroupMessage::Modal(ModifyGroupModalMessage::SubmitDelete),
                &mut vault,
                &mut modal_state,
                &mut selected_group_uuid,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyGroupView::update_modal.type_id()],
                1
            );

            assert!(call_map.borrow().values().all(|v| *v == 1));
        })
    }

    #[test]
    fn new_delete_request() {
        let modal_state = ModifyGroupModal::delete_request();

        if let ModifyGroupModal::DeleteRequest { .. } = modal_state {
        } else {
            panic!("State should be a delete request");
        }
    }

    #[test]
    fn new_group_not_empty_message() {
        let modal_state = ModifyGroupModal::group_not_empty();

        if let ModifyGroupModal::GroupNotEmpty { .. } = modal_state {
        } else {
            panic!("State should be a delete request");
        }
    }

    #[test]
    fn default_modify_group_modal() {
        let modal = ModifyGroupModal::default();
        if let ModifyGroupModal::None = modal {
        } else {
            panic!("ModifyGroupModal should be None");
        }
    }
}
