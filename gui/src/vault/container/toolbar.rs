//! TODO

use iced::{button, Element, Length, Row};

use crate::{utils::icon_button, DEFAULT_ROW_SPACING};

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

impl ToolBar {
    pub fn view(
        &mut self,
        vault_contains_unsaved_changes: bool,
        modify_entry_view_is_some: bool,
        modify_group_view_is_some: bool,
    ) -> Element<ToolBarMessage> {
        let mut save = icon_button(&mut self.save_state, "I", "Save Vault");
        if vault_contains_unsaved_changes && !modify_entry_view_is_some {
            save = save.on_press(ToolBarMessage::Save);
        }

        let mut new_group = icon_button(&mut self.new_group_state, "I", "New Group");
        let mut new_entry = icon_button(&mut self.new_entry_state, "I", "New Entry");
        if !(modify_group_view_is_some || modify_entry_view_is_some) {
            new_group = new_group.on_press(ToolBarMessage::NewGroup);
            new_entry = new_entry.on_press(ToolBarMessage::NewEntry);
        }

        let mut copy_username = icon_button(&mut self.copy_username_state, "I", "Copy Username");
        let mut copy_password = icon_button(&mut self.copy_password_state, "I", "Copy Password");
        if modify_entry_view_is_some {
            copy_username = copy_username.on_press(ToolBarMessage::CopyUsername);
            copy_password = copy_password.on_press(ToolBarMessage::CopyUsername);
        }

        let mut lock_vault = icon_button(&mut self.lock_vault_state, "I", "Lock Vault");
        if !vault_contains_unsaved_changes {
            lock_vault = lock_vault.on_press(ToolBarMessage::LockVault)
        }

        Row::with_children(vec![
            save.into(),
            new_group.into(),
            new_entry.into(),
            copy_username.into(),
            copy_password.into(),
            lock_vault.into(),
        ])
        .spacing(DEFAULT_ROW_SPACING)
        .width(Length::Fill)
        .into()
    }
}
