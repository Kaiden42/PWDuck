//! TODO

use iced::{button, Element, Length, Row};

use crate::{
    icons::Icon,
    utils::{icon_button, SomeIf},
    DEFAULT_ROW_SPACING,
};

use bitflags::bitflags;

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
    pub fn view(&mut self, flags: Flags) -> Element<ToolBarMessage> {
        let save = icon_button(
            &mut self.save_state,
            Icon::Save,
            "Save Vault",
            "Save Vault",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            ToolBarMessage::Save
                //.some_if(vault_contains_unsaved_changes && !modify_entry_view_is_some), // TODO
                .some_if(
                    flags.contains(Flags::VAULT_CONTAINS_UNSAVED_CHANGES)
                        && !flags.contains(Flags::MODIFY_ENTRY_VIEW_IS_SOME),
                ),
        );

        let new_group =
            icon_button(
                &mut self.new_group_state,
                Icon::FolderPlus,
                "New Group",
                "Create a new Group",
                flags.contains(Flags::HIDE_TOOLBAR_LABELS),
                ToolBarMessage::NewGroup.some_if_not(flags.intersects(
                    Flags::MODIFY_GROUP_VIEW_IS_SOME | Flags::MODIFY_ENTRY_VIEW_IS_SOME,
                )),
            );
        let new_entry =
            icon_button(
                &mut self.new_entry_state,
                Icon::PersonPlus,
                "New Entry",
                "Create a new Entry",
                flags.contains(Flags::HIDE_TOOLBAR_LABELS),
                ToolBarMessage::NewEntry.some_if_not(flags.intersects(
                    Flags::MODIFY_GROUP_VIEW_IS_SOME | Flags::MODIFY_ENTRY_VIEW_IS_SOME,
                )),
            );

        let autofill = icon_button(
            &mut self.auto_fill,
            Icon::Keyboard,
            "AutoType",
            "Autofill the credentials into the target window",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            ToolBarMessage::AutoFill.some_if(flags.contains(Flags::MODIFY_ENTRY_VIEW_IS_SOME)),
        );

        let lock_vault = icon_button(
            &mut self.lock_vault_state,
            Icon::Lock,
            "Lock Vault",
            "Lock Vault",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            ToolBarMessage::LockVault.some_if_not(flags.intersects(
                Flags::VAULT_CONTAINS_UNSAVED_CHANGES
                    | Flags::MODIFY_ENTRY_VIEW_IS_SOME
                    | Flags::MODIFY_GROUP_VIEW_IS_SOME,
            )),
        );

        Row::with_children(vec![save, new_group, new_entry, autofill, lock_vault])
            .spacing(DEFAULT_ROW_SPACING)
            .width(Length::Fill)
            .into()
    }
}

bitflags! {
    /// The configuration of the [`ToolBar`](ToolBar).
    pub struct Flags: u8 {
        const VAULT_CONTAINS_UNSAVED_CHANGES = 0b1;
        const MODIFY_ENTRY_VIEW_IS_SOME = 0b1 << 1;
        const MODIFY_GROUP_VIEW_IS_SOME = 0b1 << 2;
        const HIDE_TOOLBAR_LABELS = 0b1 << 3;
    }
}
