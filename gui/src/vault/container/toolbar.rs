//! TODO

use iced::{button, Element, Length, Row};

use crate::{
    icons::Icon,
    theme::Theme,
    utils::{icon_button, ButtonData, ButtonKind, SomeIf},
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
    pub fn view(&mut self, flags: Flags, theme: &dyn Theme) -> Element<ToolBarMessage> {
        let save = icon_button(
            ButtonData {
                state: &mut self.save_state,
                icon: Icon::Save,
                text: "Save vault",
                kind: if flags.contains(Flags::VAULT_CONTAINS_UNSAVED_CHANGES) {
                    ButtonKind::Primary
                } else {
                    ButtonKind::Normal
                },
                on_press: ToolBarMessage::Save.some_if(
                    flags.contains(Flags::VAULT_CONTAINS_UNSAVED_CHANGES)
                        && !flags.contains(Flags::MODIFY_ENTRY_VIEW_IS_SOME),
                ),
            },
            "Save vault",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            theme,
        );

        let new_group = icon_button(
            ButtonData {
                state: &mut self.new_group_state,
                icon: Icon::FolderPlus,
                text: "New group",
                kind: ButtonKind::Normal,
                on_press: ToolBarMessage::NewGroup.some_if_not(flags.intersects(
                    Flags::MODIFY_GROUP_VIEW_IS_SOME | Flags::MODIFY_ENTRY_VIEW_IS_SOME,
                )),
            },
            "Create a new group",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            theme,
        );

        let new_entry = icon_button(
            ButtonData {
                state: &mut self.new_entry_state,
                icon: Icon::PersonPlus,
                text: "New entry",
                kind: ButtonKind::Normal,
                on_press: ToolBarMessage::NewEntry.some_if_not(flags.intersects(
                    Flags::MODIFY_GROUP_VIEW_IS_SOME | Flags::MODIFY_ENTRY_VIEW_IS_SOME,
                )),
            },
            "Create a new entry",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            theme,
        );

        let autofill = icon_button(
            ButtonData {
                state: &mut self.auto_fill,
                icon: Icon::Keyboard,
                text: "AutoType",
                kind: ButtonKind::Normal,
                on_press: ToolBarMessage::AutoFill
                    .some_if(flags.contains(Flags::MODIFY_ENTRY_VIEW_IS_SOME)),
            },
            "Autofill the credentials into the target window",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            theme,
        );

        let lock_vault = icon_button(
            ButtonData {
                state: &mut self.lock_vault_state,
                icon: Icon::Lock,
                text: "Lock vault",
                kind: ButtonKind::Normal,
                on_press: ToolBarMessage::LockVault.some_if_not(flags.intersects(
                    Flags::VAULT_CONTAINS_UNSAVED_CHANGES
                        | Flags::MODIFY_ENTRY_VIEW_IS_SOME
                        | Flags::MODIFY_GROUP_VIEW_IS_SOME,
                )),
            },
            "Lock vault",
            flags.contains(Flags::HIDE_TOOLBAR_LABELS),
            theme,
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
