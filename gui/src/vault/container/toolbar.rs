//! TODO

use iced::{button, Element, Length, Row};

use crate::{
    icons::Icon,
    utils::{icon_button, SomeIf},
    DEFAULT_ROW_SPACING,
};

/// The state of the toolbar.
#[derive(Debug, Default)]
pub struct ToolBar {
    /// The state of the save [`Button`](iced::Button).
    save_state: button::State,
    /// The state of the [`Button`](iced::Button) to create a new group.
    new_group_state: button::State,
    /// The state of the [`Button`](iced:Button) to create a new entry.
    new_entry_state: button::State,
    /// The state of the autofill [`Button`](Button)
    auto_fill: button::State,
    /// The state of the lock [`Button`](iced::Button).
    lock_vault_state: button::State,
}

/// The message that is send by the toolbar.
#[derive(Clone, Debug)]
pub enum ToolBarMessage {
    /// Save the vault.
    Save,
    /// Create a new group.
    NewGroup,
    /// Create a new entry.
    NewEntry,
    /// Autofill the credentials.
    AutoFill,
    /// Lock the vault.
    LockVault,
}
impl SomeIf for ToolBarMessage {}

impl ToolBar {
    /// Create the view of the [`ToolBar`](ToolBar).
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn view(
        &mut self,
        vault_contains_unsaved_changes: bool,
        modify_entry_view_is_some: bool,
        modify_group_view_is_some: bool,
        hide_toolbar_labels: bool,
    ) -> Element<ToolBarMessage> {
        let save = icon_button(
            &mut self.save_state,
            Icon::Save,
            "Save Vault",
            "Save Vault",
            hide_toolbar_labels,
            ToolBarMessage::Save
                .some_if(vault_contains_unsaved_changes && !modify_entry_view_is_some),
        );

        let new_group = icon_button(
            &mut self.new_group_state,
            Icon::FolderPlus,
            "New Group",
            "Create a new Group",
            hide_toolbar_labels,
            ToolBarMessage::NewGroup
                .some_if_not(modify_group_view_is_some || modify_entry_view_is_some),
        );
        let new_entry = icon_button(
            &mut self.new_entry_state,
            Icon::PersonPlus,
            "New Entry",
            "Create a new Entry",
            hide_toolbar_labels,
            ToolBarMessage::NewEntry
                .some_if_not(modify_group_view_is_some || modify_entry_view_is_some),
        );

        let autofill = icon_button(
            &mut self.auto_fill,
            Icon::Gear, // TODO
            "Autofill",
            "Autofill the credentials into the target window",
            hide_toolbar_labels,
            ToolBarMessage::AutoFill.some_if(false),
        );

        let lock_vault = icon_button(
            &mut self.lock_vault_state,
            Icon::Lock,
            "Lock Vault",
            "Lock Vault",
            hide_toolbar_labels,
            ToolBarMessage::LockVault.some_if_not(
                vault_contains_unsaved_changes
                    || modify_entry_view_is_some
                    || modify_group_view_is_some,
            ),
        );

        Row::with_children(vec![save, new_group, new_entry, autofill, lock_vault])
            .spacing(DEFAULT_ROW_SPACING)
            .width(Length::Fill)
            .into()
    }
}
