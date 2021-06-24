//! TODO

use iced::{button, Element, Length, Row};

use crate::{
    icons::Icon,
    utils::{icon_button, SomeIf},
    DEFAULT_ROW_SPACING,
};

/// TODO
#[derive(Debug, Default)]
pub struct ToolBar {
    /// TODO
    save_state: button::State,
    /// TODO
    new_group_state: button::State,
    /// TODO
    new_entry_state: button::State,
    /// TODO
    copy_username_state: button::State,
    /// TODO
    copy_password_state: button::State,
    /// TODO
    lock_vault_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum ToolBarMessage {
    /// TODO
    Save,
    /// TODO
    NewGroup,
    /// TODO
    NewEntry,
    /// TODO
    CopyUsername,
    /// TODO
    CopyPassword,
    /// TODO
    LockVault,
}
impl SomeIf for ToolBarMessage {}

impl ToolBar {
    /// TODO
    pub fn view(
        &mut self,
        vault_contains_unsaved_changes: bool,
        modify_entry_view_is_some: bool,
        modify_group_view_is_some: bool,
    ) -> Element<ToolBarMessage> {
        let save = icon_button(
            &mut self.save_state,
            Icon::Save,
            "Save Vault",
            "Save Vault",
            false,
            ToolBarMessage::Save
                .some_if(vault_contains_unsaved_changes && !modify_entry_view_is_some),
        );

        let new_group = icon_button(
            &mut self.new_group_state,
            Icon::FolderPlus,
            "New Group",
            "Create a new Group",
            false,
            ToolBarMessage::NewGroup
                .some_if_not(modify_group_view_is_some || modify_entry_view_is_some),
        );
        let new_entry = icon_button(
            &mut self.new_entry_state,
            Icon::PersonPlus,
            "New Entry",
            "Create a new Entry",
            false,
            ToolBarMessage::NewEntry
                .some_if_not(modify_group_view_is_some || modify_entry_view_is_some),
        );

        let copy_username = icon_button(
            &mut self.copy_username_state,
            Icon::FileEarmarkPerson,
            "C. Username",
            "Copy Username to clipboard",
            false,
            ToolBarMessage::CopyUsername.some_if(modify_entry_view_is_some),
        );
        let copy_password = icon_button(
            &mut self.copy_password_state,
            Icon::FileEarmarkLock,
            "C. Password",
            "Copy Password to clipboard",
            false,
            ToolBarMessage::CopyUsername.some_if(modify_entry_view_is_some),
        );

        let lock_vault = icon_button(
            &mut self.lock_vault_state,
            Icon::Lock,
            "Lock Vault",
            "Lock Vault",
            false,
            ToolBarMessage::LockVault.some_if_not(
                vault_contains_unsaved_changes
                    || modify_entry_view_is_some
                    || modify_group_view_is_some,
            ),
        );

        Row::with_children(vec![
            save,
            new_group,
            new_entry,
            copy_username,
            copy_password,
            lock_vault,
        ])
        .spacing(DEFAULT_ROW_SPACING)
        .width(Length::Fill)
        .into()
    }
}
