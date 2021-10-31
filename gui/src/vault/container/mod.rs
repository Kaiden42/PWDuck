//! The view of the content of a vault.
use std::sync::MutexGuard;

use iced::{Column, Command, Container, Length};
use iced_focus::Focus;
use pwduck_core::{AutoTypeSequenceParser, EntryBody, EntryHead, Group, MemKey, Uuid, Vault};

mod list;
use list::{ListMessage, ListView};

mod modify_entry;
use modify_entry::ModifyEntryView;
pub use modify_entry::{ModifyEntryMessage, ModifyEntryModal};

mod modify_group;
use modify_group::ModifyGroupView;
pub use modify_group::{ModifyGroupMessage, ModifyGroupModal};

use getset::Getters;

mod toolbar;
use toolbar::ToolBar;
pub use toolbar::ToolBarMessage;

use crate::{
    error::PWDuckGuiError, theme::Theme, utils::default_vertical_space, Component, Platform,
    Viewport, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING,
};

use self::list::ListItemMessage;

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the vault container.
#[derive(Debug, Getters, Focus)]
pub struct VaultContainer {
    /// The unlocked vault.
    #[getset(get = "pub")]
    vault: Box<Vault>,

    /// The state of the [`ToolBar`](ToolBar).
    tool_bar: ToolBar,

    /// The state of the current view.
    current_view: CurrentView,

    /// The state of the list view.
    #[focus(enable = "self.enable_list_view_focus")]
    list_view: ListView,

    /// The state of the group modification view.
    #[getset(get = "pub")]
    #[focus(enable = "self.enable_modify_group_view_focus")]
    modify_group_view: Option<Box<ModifyGroupView>>,

    /// The state of the entry modification view.
    #[getset(get = "pub")]
    #[focus(enable = "self.enable_modify_entry_view_focus")]
    modify_entry_view: Option<Box<ModifyEntryView>>,
}

#[cfg_attr(test, mockable)]
impl VaultContainer {
    /// True, if the container contains unsaved changes.
    #[must_use]
    pub fn contains_unsaved_changes(&self) -> bool {
        self.vault.contains_unsaved_changes()
            || self
                .modify_group_view
                .as_ref()
                .map_or(false, |view| view.contains_unsaved_changes())
            || self
                .modify_entry_view
                .as_ref()
                .map_or(false, |view| view.contains_unsaved_changes())
    }

    /// If the [`Focus`](iced_focus::Focus) of the [`ListView`](ListView) is enabled.
    const fn enable_list_view_focus(&self) -> bool {
        self.modify_group_view.is_none() && self.modify_entry_view.is_none()
    }

    /// If the [`Focus`](iced_focus::Focus) of the [`ModifyGroupView`](ModifyGroupView) is enabled.
    const fn enable_modify_group_view_focus(&self) -> bool {
        self.modify_group_view.is_some()
    }

    /// If the [`Focus`](iced_focus::Focus) of the [`ModifyEntryView`](ModifyEntryView) is enabled.
    const fn enable_modify_entry_view_focus(&self) -> bool {
        self.modify_entry_view.is_some()
    }

    /// Save the vault to disk.
    fn save(
        &mut self,
        mem_key: &MutexGuard<MemKey>,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        // TODO: find a way to do this async
        self.vault.save(mem_key)?;
        Ok(Command::none())
    }

    /// Create a new group and switch to the [`ModifyGroupView`](ModifyGroupView) as the current view.
    fn create_group(&mut self) -> Command<VaultContainerMessage> {
        let group = Group::new(
            pwduck_core::Uuid::new(self.vault.path()),
            self.list_view.selected_group_uuid().clone(),
            String::new(),
        );

        self.modify_group_view = Some(Box::new(ModifyGroupView::with(
            modify_group::State::Create,
            group,
        )));
        self.current_view = CurrentView::ModifyGroup;
        Command::none()
    }

    /// Create a new entry and switch to the [`ModifyEntryView`](ModifyEntryView) as the current view.
    fn create_entry(&mut self) -> Command<VaultContainerMessage> {
        let entry_body = EntryBody::new(
            pwduck_core::Uuid::new(self.vault.path()),
            String::new(),
            String::new(),
        );
        let entry_head = EntryHead::new(
            pwduck_core::Uuid::new(self.vault.path()),
            self.list_view.selected_group_uuid().clone(),
            String::new(),
            entry_body.uuid().clone(),
        );

        self.modify_entry_view = Some(Box::new(ModifyEntryView::with(
            modify_entry::State::Create,
            entry_head,
            entry_body,
        )));
        self.current_view = CurrentView::ModifyEntry;

        Command::none()
    }

    /// Copy the username to the clipboard.
    #[cfg_attr(coverage, no_coverage)]
    fn copy_username(
        &self,
        uuid: &Uuid,
        mem_key: &MutexGuard<MemKey>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let master_key = self.vault.master_key().as_unprotected(
            mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        let entry_body = self.vault.unsaved_entry_bodies().get(uuid).map_or_else(
            || pwduck_core::EntryBody::load(self.vault.path(), uuid, &master_key),
            |dto| pwduck_core::EntryBody::decrypt(dto, &master_key),
        )?;

        clipboard.write(entry_body.username().to_string());

        Ok(Command::none())
    }

    /// Copy the password to the clipboard.
    #[cfg_attr(coverage, no_coverage)]
    fn copy_password(
        &self,
        uuid: &Uuid,
        mem_key: &MutexGuard<MemKey>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let master_key = self.vault.master_key().as_unprotected(
            mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        let entry_body = self.vault.unsaved_entry_bodies().get(uuid).map_or_else(
            || pwduck_core::EntryBody::load(self.vault.path(), uuid, &master_key),
            |dto| pwduck_core::EntryBody::decrypt(dto, &master_key),
        )?;

        clipboard.write(entry_body.password().to_string());

        Ok(Command::none())
    }

    /// Update the [`ToolBar`](ToolBar) with the given message.
    fn update_toolbar<P: Platform + 'static>(
        &mut self,
        message: &ToolBarMessage,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ToolBarMessage::Save => self.save(&crate::MEM_KEY.lock()?),
            ToolBarMessage::NewGroup => Ok(self.create_group()),
            ToolBarMessage::NewEntry => Ok(self.create_entry()),
            ToolBarMessage::AutoFill => self.modify_entry_view.as_ref().map_or_else(
                || Ok(Command::none()),
                |view| self.auto_fill::<P>(view.entry_head().uuid(), &crate::MEM_KEY.lock()?),
            ),
            ToolBarMessage::LockVault => {
                PWDuckGuiError::Unreachable("ToolBarMessage".into()).into()
            }
        }
    }

    /// Update the search and replace it with the given value. The [`ListView`](ListView) will be resized.
    fn update_search(&mut self, search: String) -> Command<VaultContainerMessage> {
        let _ = self.list_view.set_search(search);
        self.list_view.resize(&self.vault);
        Command::none()
    }

    /// Go back to the parent group of the currently selected group of the unlocked vault.
    fn go_to_parent_group(&mut self) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let group = self
            .vault
            .groups()
            .get(self.list_view.selected_group_uuid())
            .ok_or(PWDuckGuiError::Option)?;

        if let Some(group_uuid) = group.parent() {
            let _ = self.list_view.set_selected_group_uuid(group_uuid.clone());
            self.list_view.resize(&self.vault);
        }
        Ok(Command::none())
    }

    /// Edit the currently selected group of the unlocked vault.
    fn edit_group(&mut self) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let group = self
            .vault
            .groups()
            .get(self.list_view.selected_group_uuid())
            .ok_or(PWDuckGuiError::Option)?
            .clone();

        if group.is_root() {
            return Err(PWDuckGuiError::Unreachable(
                "Root should not be able to edit".into(),
            ));
        }

        self.modify_group_view = Some(Box::new(ModifyGroupView::with(
            modify_group::State::Modify,
            group,
        )));
        self.current_view = CurrentView::ModifyGroup;
        Ok(Command::none())
    }

    /// Select the group identified by the UUID.
    fn select_group(&mut self, uuid: Uuid) -> Command<VaultContainerMessage> {
        let _ = self.list_view.set_selected_group_uuid(uuid);
        self.list_view.search_mut().clear();
        self.list_view.resize(&self.vault);
        Command::none()
    }

    /// Select the entry identified by the UUID. It will be loaded, decrypted
    /// and finally displayed in the [`ModifyEntryView`](ModifyEntryView).
    fn select_entry(
        &mut self,
        uuid: &Uuid,
        mem_key: &MutexGuard<MemKey>,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let entry_head = self
            .vault
            .entries()
            .get(uuid)
            .ok_or(PWDuckGuiError::Option)?
            .clone();

        let master_key = self.vault.master_key().as_unprotected(
            mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        // Load body from cache if exists, otherwise load from file system.
        let entry_body = self
            .vault
            .unsaved_entry_bodies()
            .get(entry_head.body())
            .map_or_else(
                || pwduck_core::EntryBody::load(self.vault.path(), entry_head.body(), &master_key),
                |dto| pwduck_core::EntryBody::decrypt(dto, &master_key),
            )?;

        self.modify_entry_view = Some(Box::new(ModifyEntryView::with(
            modify_entry::State::Modify,
            entry_head,
            entry_body,
        )));
        self.current_view = CurrentView::ModifyEntry;
        Ok(Command::none())
    }

    /// Autotype the credentials of the entry identified by it's UUID.
    fn auto_fill<P: Platform + 'static>(
        &self,
        uuid: &Uuid,
        mem_key: &MutexGuard<MemKey>,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let entry_head = self
            .vault
            .entries()
            .get(uuid)
            .ok_or(PWDuckGuiError::Option)?;

        let master_key = self.vault.master_key().as_unprotected(
            mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        let entry_body = self
            .vault
            .unsaved_entry_bodies()
            .get(entry_head.body())
            .map_or_else(
                || pwduck_core::EntryBody::load(self.vault.path(), entry_head.body(), &master_key),
                |dto| pwduck_core::EntryBody::decrypt(dto, &master_key),
            )?;

        let sequence = AutoTypeSequenceParser::parse_sequence(
            entry_head.auto_type_sequence(),
            entry_head,
            &entry_body,
        )?;

        Ok(Command::perform(
            P::auto_type(sequence),
            VaultContainerMessage::AutoTypeResult,
        ))
    }

    /// Resizes the split panel to the given divider position.
    fn split_resize(&mut self, position: u16) -> Command<VaultContainerMessage> {
        self.list_view
            .split_state_mut()
            .set_divider_position(position);
        Command::none()
    }

    /// Handle the message that was send by the group tree.
    fn update_group_tree(
        &mut self,
        message: list::GroupTreeMessage,
    ) -> Result<Command<list::GroupTreeMessage>, PWDuckGuiError> {
        match message {
            list::GroupTreeMessage::ToggleExpansion(_) => {
                self.list_view.group_tree_mut().update(message, &self.vault)
            }
            list::GroupTreeMessage::GroupSelected(uuid) => {
                let _ = self.list_view.set_selected_group_uuid(uuid);
                self.list_view.resize(&self.vault);
                Ok(Command::none())
            }
        }
    }

    /// Handle the message that was send by the list items.
    fn update_list_items<P: Platform + 'static>(
        &mut self,
        message: ListItemMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ListItemMessage::GroupSelected(uuid) => Ok(self.select_group(uuid)),
            ListItemMessage::EntrySelected(uuid) => {
                self.select_entry(&uuid, &crate::MEM_KEY.lock()?)
            }
            ListItemMessage::CopyUsername(uuid) => {
                self.copy_username(&uuid, &crate::MEM_KEY.lock()?, clipboard)
            }
            ListItemMessage::CopyPassword(uuid) => {
                self.copy_password(&uuid, &crate::MEM_KEY.lock()?, clipboard)
            }
            ListItemMessage::Autofill(uuid) => self.auto_fill::<P>(&uuid, &crate::MEM_KEY.lock()?),
        }
    }

    /// Handle the message that was send by the [`ListView`](ListView).
    fn update_list<P: Platform + 'static>(
        &mut self,
        message: ListMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ListMessage::SearchInput(search) => Ok(self.update_search(search)),
            ListMessage::Back => self.go_to_parent_group(),
            ListMessage::EditGroup => self.edit_group(),
            ListMessage::ListItemMessage(message) => {
                self.update_list_items::<P>(message, clipboard)
            }
            ListMessage::SplitResize(position) => Ok(self.split_resize(position)),
            ListMessage::GroupTreeMessage(message) => Ok(self
                .update_group_tree(message)?
                .map(ListMessage::GroupTreeMessage)
                .map(VaultContainerMessage::List)),
        }
    }

    /// Handle the massage that was send by the [`ModifyGroupView`](ModifyGroupView).
    fn update_modify_group(
        &mut self,
        message: &ModifyGroupMessage,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let vault = &mut self.vault;
        let selected_group_uuid = self.list_view.selected_group_uuid_mut();

        let cmd = self
            .modify_group_view
            .as_mut()
            .map_or_else(
                || Ok(Command::none()),
                |view| {
                    view.update(
                        message.clone(),
                        vault,
                        modal_state,
                        selected_group_uuid,
                        clipboard,
                    )
                },
            )
            .map(|cmd| cmd.map(VaultContainerMessage::ModifyGroup));

        match message {
            ModifyGroupMessage::Cancel
            | ModifyGroupMessage::Submit
            | &ModifyGroupMessage::Modal(modify_group::ModifyGroupModalMessage::SubmitDelete) => {
                self.list_view.resize(&self.vault);
                self.list_view.group_tree_mut().refresh(&self.vault);
                self.current_view = CurrentView::ListView;
                self.modify_group_view = None;
            }
            _ => {}
        }

        cmd
    }

    /// Handle the message that was send by the [`ModifyEntryView`](ModifyEntryView).
    fn update_modify_entry<P: Platform + 'static>(
        &mut self,
        message: &ModifyEntryMessage,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let vault = &mut self.vault;
        let cmd = self
            .modify_entry_view
            .as_mut()
            .map_or_else(
                || Ok(Command::none()),
                |view| view.update::<P>(message.clone(), vault, modal_state, clipboard),
            )
            .map(|cmd| cmd.map(VaultContainerMessage::ModifyEntry));

        match message {
            ModifyEntryMessage::Cancel
            | ModifyEntryMessage::Submit
            | ModifyEntryMessage::Modal(modify_entry::ModifyEntryModalMessage::SubmitDelete) => {
                self.current_view = CurrentView::ListView;
                self.modify_entry_view = None;
                self.list_view.resize(&self.vault);
            }
            _ => {}
        }

        cmd
    }
}

/// The current view to display.
#[derive(Debug, PartialEq)]
enum CurrentView {
    /// Display the [`ListView`](ListView).
    ListView,
    /// Display the [`ModifyGroupView`](ModifyGroupView).
    ModifyGroup,
    /// Display the [`ModifyEntryView`](ModifyEntryView).
    ModifyEntry,
}

/// The message that is send by the vault container.
#[derive(Clone, Debug)]
pub enum VaultContainerMessage {
    /// The message that is send by the ToolBar.
    ToolBar(ToolBarMessage),
    /// The message that is send by the ListView`.
    List(ListMessage),
    /// The message that is send by the ModifyGroupView.
    ModifyGroup(ModifyGroupMessage),
    /// The message that is send by the ModifyEntryView.
    ModifyEntry(ModifyEntryMessage),
    /// The result of the autotyper.
    AutoTypeResult(Result<(), PWDuckGuiError>),
}

#[cfg_attr(test, mockable)]
impl Component for VaultContainer {
    type Message = VaultContainerMessage;
    type ConstructorParam = Box<Vault>;

    fn new(vault: Self::ConstructorParam) -> Self {
        let root_uuid = vault.get_root_uuid().unwrap();
        let list_view = ListView::new(root_uuid, &vault);

        Self {
            vault,
            tool_bar: ToolBar::default(),
            current_view: CurrentView::ListView,
            list_view,
            modify_group_view: None,
            modify_entry_view: None,
        }
    }

    fn title(&self) -> String {
        if self.vault.contains_unsaved_changes() {
            format!("(\u{2731}) {}", self.vault.get_name())
        } else {
            self.vault.get_name().to_owned()
        }
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _application_settings: &mut pwduck_core::ApplicationSettings,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<Self::Message>, PWDuckGuiError> {
        match message {
            VaultContainerMessage::ToolBar(message) => {
                self.update_toolbar::<P>(&message, clipboard)
            }

            VaultContainerMessage::List(message) => self.update_list::<P>(message, clipboard),

            VaultContainerMessage::ModifyGroup(message) => {
                self.update_modify_group(&message, modal_state, clipboard)
            }

            VaultContainerMessage::ModifyEntry(message) => {
                self.update_modify_entry::<P>(&message, modal_state, clipboard)
            }

            VaultContainerMessage::AutoTypeResult(result) => {
                result?;
                Ok(Command::none())
            }
        }
    }

    #[cfg_attr(coverage, no_coverage)]
    fn view<P: Platform + 'static>(
        &mut self,
        _application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message> {
        let mut flags = toolbar::Flags::empty();
        flags.set(
            toolbar::Flags::VAULT_CONTAINS_UNSAVED_CHANGES,
            self.vault.contains_unsaved_changes(),
        );
        flags.set(
            toolbar::Flags::MODIFY_ENTRY_VIEW_IS_SOME,
            self.modify_entry_view.is_some(),
        );
        flags.set(
            toolbar::Flags::MODIFY_GROUP_VIEW_IS_SOME,
            self.modify_group_view.is_some(),
        );
        flags.set(toolbar::Flags::HIDE_TOOLBAR_LABELS, viewport.width < 800);

        let tool_bar = self
            .tool_bar
            .view(flags, theme)
            .map(VaultContainerMessage::ToolBar);

        let body = match self.current_view {
            CurrentView::ListView => self
                .list_view
                .view(&self.vault, theme, viewport)
                .map(VaultContainerMessage::List),

            CurrentView::ModifyGroup => match &mut self.modify_group_view {
                Some(modify_group_view) => modify_group_view
                    .view(&self.vault, self.list_view.selected_group_uuid(), theme)
                    .map(VaultContainerMessage::ModifyGroup),
                None => unreachable!(),
            },

            CurrentView::ModifyEntry => match &mut self.modify_entry_view {
                Some(modify_enty_view) => modify_enty_view
                    .view::<P>(self.list_view.selected_group_uuid(), theme)
                    .map(VaultContainerMessage::ModifyEntry),
                None => unreachable!(),
            },
        };

        Container::new(
            Column::new()
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_COLUMN_SPACING)
                .push(tool_bar)
                .push(default_vertical_space())
                .push(body),
        )
        .style(theme.container())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
        sync::Mutex,
    };

    use iced::Command;
    use mocktopus::mocking::*;
    use modify_entry::{ModifyEntryMessage, ModifyEntryModalMessage};
    use modify_group::{ModifyGroupMessage, ModifyGroupModalMessage};
    use pwduck_core::{uuid, EntryHead, MemKey, Vault};
    use tempfile::{tempdir, TempDir};

    use crate::{error::PWDuckGuiError, Component, TestPlatform};

    use super::{
        list::{GroupTree, GroupTreeMessage, ListItemMessage, ListMessage, ListView},
        modify_entry::{self, ModifyEntryView},
        modify_group::{self, ModifyGroupView},
        CurrentView, ToolBarMessage, VaultContainer, VaultContainerMessage,
    };

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    const PASSWORD: &str = "this is a totally secret password";
    const DEFAULT_GROUP_COUNT: u8 = 15;
    const DEFAULT_ENTRY_COUNT: u8 = 15;

    fn default_vault(mem_key: &MemKey) -> (TempDir, Vault) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");
        let mut vault =
            pwduck_core::Vault::generate(PASSWORD, Option::<String>::None, mem_key, &path).unwrap();
        let master_key = vault
            .master_key()
            .as_unprotected(mem_key, vault.salt(), vault.nonce())
            .unwrap();
        let root = vault.get_root_uuid().unwrap();

        // Add 10 groups
        for i in 0..DEFAULT_GROUP_COUNT {
            let group = pwduck_core::Group::new(
                [i; uuid::SIZE].into(),
                root.clone(),
                format!("Group: {}", i),
            );
            vault.insert_group(group);
        }

        // Add 10 entries
        for i in 0..DEFAULT_ENTRY_COUNT {
            let head = pwduck_core::EntryHead::new(
                [i; uuid::SIZE].into(),
                root.clone(),
                format!("Entry: {}", i),
                [i; uuid::SIZE].into(),
            );
            let body = pwduck_core::EntryBody::new(
                [i; uuid::SIZE].into(),
                "username".into(),
                "password".into(),
            );
            vault.insert_entry(head, body, &master_key).unwrap();
        }

        (dir, vault)
    }

    #[test]
    fn contains_unsaved_changes() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        ModifyGroupView::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));
        ModifyEntryView::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));

        let mut vault_container = VaultContainer::new(Box::new(vault));

        // The default vault should contain unsaved changes.
        assert!(vault_container.contains_unsaved_changes());

        vault_container.vault.save(&mem_key).unwrap();

        // After save the vault should not contain unsaved changes.
        assert!(!vault_container.contains_unsaved_changes());

        ModifyGroupView::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(true));
        // A non existent group view should not contain unsaved changes
        assert!(!vault_container.contains_unsaved_changes());
        vault_container.modify_group_view = Some(Box::new(ModifyGroupView::with(
            modify_group::State::Create,
            pwduck_core::Group::new([1; uuid::SIZE].into(), root.clone(), "".into()),
        )));
        // An existent group view should contain unsaved changes
        assert!(vault_container.contains_unsaved_changes());
        ModifyGroupView::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));
        assert!(!vault_container.contains_unsaved_changes());

        ModifyEntryView::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(true));
        // A non existent entry view should not contain unsaved changes
        assert!(!vault_container.contains_unsaved_changes());
        vault_container.modify_entry_view = Some(Box::new(ModifyEntryView::with(
            modify_entry::State::Create,
            pwduck_core::EntryHead::new(
                [1; uuid::SIZE].into(),
                root.clone(),
                "".into(),
                [2; uuid::SIZE].into(),
            ),
            pwduck_core::EntryBody::new([2; uuid::SIZE].into(), "".into(), "".into()),
        )));
        // An existent entry view should contain unsaved changes
        assert!(vault_container.contains_unsaved_changes());
        ModifyEntryView::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));
        assert!(!vault_container.contains_unsaved_changes());
    }

    #[test]
    fn enable_view_focus() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut vault_container = VaultContainer::new(Box::new(vault));
        assert!(vault_container.modify_group_view.is_none());
        assert!(vault_container.modify_entry_view.is_none());
        assert!(vault_container.enable_list_view_focus());
        assert!(!vault_container.enable_modify_group_view_focus());
        assert!(!vault_container.enable_modify_entry_view_focus());

        vault_container.modify_group_view = Some(Box::new(ModifyGroupView::with(
            modify_group::State::Create,
            pwduck_core::Group::new([1; uuid::SIZE].into(), root.clone(), "".into()),
        )));
        assert!(vault_container.modify_group_view.is_some());
        assert!(!vault_container.enable_list_view_focus());
        assert!(vault_container.enable_modify_group_view_focus());
        assert!(!vault_container.enable_modify_entry_view_focus());

        vault_container.modify_group_view = None;
        vault_container.modify_entry_view = Some(Box::new(ModifyEntryView::with(
            modify_entry::State::Create,
            pwduck_core::EntryHead::new(
                [1; uuid::SIZE].into(),
                root.clone(),
                "".into(),
                [2; uuid::SIZE].into(),
            ),
            pwduck_core::EntryBody::new([2; uuid::SIZE].into(), "".into(), "".into()),
        )));
        assert!(vault_container.modify_entry_view.is_some());
        assert!(!vault_container.enable_list_view_focus());
        assert!(!vault_container.enable_modify_group_view_focus());
        assert!(vault_container.enable_modify_entry_view_focus());
    }

    #[test]
    fn save() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);

        let mut vault_container = VaultContainer::new(Box::new(vault));

        let mutex_mem_key = Mutex::new(mem_key);

        assert!(vault_container.vault.contains_unsaved_changes());
        let _ = vault_container
            .save(&mutex_mem_key.lock().unwrap())
            .expect("Should not fail");
        assert!(!vault_container.vault.contains_unsaved_changes());
    }

    #[test]
    fn create_group() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);

        let mut vault_container = VaultContainer::new(Box::new(vault));

        assert!(vault_container.modify_group_view.is_none());
        assert_eq!(vault_container.current_view, CurrentView::ListView);
        let _ = vault_container.create_group();
        assert!(vault_container.modify_group_view.is_some());
        assert_eq!(vault_container.current_view, CurrentView::ModifyGroup);

        let modify_group_view = vault_container.modify_group_view().as_ref().unwrap();
        assert_eq!(
            modify_group_view
                .group()
                .parent()
                .as_ref()
                .expect("Group should have a parent"),
            vault_container.list_view.selected_group_uuid()
        );
        assert_eq!(modify_group_view.group().title().as_str(), "");
    }

    #[test]
    fn create_entry() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);

        let mut vault_container = VaultContainer::new(Box::new(vault));

        assert!(vault_container.modify_entry_view.is_none());
        assert_eq!(vault_container.current_view, CurrentView::ListView);
        let _ = vault_container.create_entry();
        assert!(vault_container.modify_entry_view.is_some());
        assert_eq!(vault_container.current_view, CurrentView::ModifyEntry);

        let modify_entry_view = vault_container.modify_entry_view().as_ref().unwrap();
        assert_eq!(
            modify_entry_view.entry_head().parent(),
            vault_container.list_view.selected_group_uuid()
        );
        assert_eq!(modify_entry_view.entry_head().title().as_str(), "");
        assert_eq!(
            modify_entry_view.entry_head().body(),
            modify_entry_view.entry_body().uuid()
        );
        assert_eq!(modify_entry_view.entry_body().username().as_str(), "");
        assert_eq!(modify_entry_view.entry_body().password().as_str(), "");
    }

    #[test]
    fn update_toolbar() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultContainer::save.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::create_group.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::create_entry.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::auto_fill::<TestPlatform>.type_id(), 0);

            VaultContainer::save.mock_raw(|_self, _mem_key| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::save.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::create_group.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::create_group.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultContainer::create_entry.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::create_entry.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultContainer::auto_fill::<TestPlatform>.mock_raw(|_self, _uuid, _mem_key| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::auto_fill::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Save
            assert_eq!(call_map.borrow()[&VaultContainer::save.type_id()], 0);
            let _ = vault_container
                .update_toolbar::<TestPlatform>(&ToolBarMessage::Save, &mut clipboard);
            assert_eq!(call_map.borrow()[&VaultContainer::save.type_id()], 1);

            // New group
            assert_eq!(
                call_map.borrow()[&VaultContainer::create_group.type_id()],
                0
            );
            let _ = vault_container
                .update_toolbar::<TestPlatform>(&ToolBarMessage::NewGroup, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&VaultContainer::create_group.type_id()],
                1
            );

            // New entry
            assert_eq!(
                call_map.borrow()[&VaultContainer::create_entry.type_id()],
                0
            );
            let _ = vault_container
                .update_toolbar::<TestPlatform>(&ToolBarMessage::NewEntry, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&VaultContainer::create_entry.type_id()],
                1
            );

            // Auto fill
            assert_eq!(
                call_map.borrow()[&VaultContainer::auto_fill::<TestPlatform>.type_id()],
                0
            );
            // A non existent ModifyEntryView can not auto fill.
            let _ = vault_container
                .update_toolbar::<TestPlatform>(&ToolBarMessage::AutoFill, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&VaultContainer::auto_fill::<TestPlatform>.type_id()],
                0
            );
            // An existent ModifyEntryView can auto fill.
            vault_container.modify_entry_view = Some(Box::new(ModifyEntryView::with(
                modify_entry::State::Create,
                pwduck_core::EntryHead::new(
                    [1; uuid::SIZE].into(),
                    root.clone(),
                    "".into(),
                    [2; uuid::SIZE].into(),
                ),
                pwduck_core::EntryBody::new([2; uuid::SIZE].into(), "".into(), "".into()),
            )));
            let _ = vault_container
                .update_toolbar::<TestPlatform>(&ToolBarMessage::AutoFill, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&VaultContainer::auto_fill::<TestPlatform>.type_id()],
                1
            );

            // Lock vault
            let res = vault_container
                .update_toolbar::<TestPlatform>(&ToolBarMessage::LockVault, &mut clipboard)
                .expect_err("Should fail");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }
        })
    }

    #[test]
    fn update_search() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map.borrow_mut().insert(ListView::resize.type_id(), 0);

            let search_string = "This is my search string";

            ListView::resize.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ListView::resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            assert_eq!(vault_container.list_view.search().as_str(), "");
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            let _ = vault_container.update_search(search_string.to_owned());
            assert_eq!(vault_container.list_view.search().as_str(), search_string);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
        });
    }

    #[test]
    fn go_to_parent_group() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map.borrow_mut().insert(ListView::resize.type_id(), 0);

            ListView::resize.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ListView::resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            // Root should be go to root. No resize required.
            assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            let _ = vault_container
                .go_to_parent_group()
                .expect("Should not fail.");
            assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);

            // Set selected group to the third children of root
            let roots_3rd_children_uuid = vault_container
                .vault
                .get_groups_of(&root)
                .get(3)
                .unwrap()
                .uuid()
                .to_owned();
            let _ = vault_container
                .list_view
                .set_selected_group_uuid(roots_3rd_children_uuid.clone());

            // Childrens parent should be root. Resize is required.
            assert_eq!(
                vault_container.list_view.selected_group_uuid(),
                &roots_3rd_children_uuid
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            let _ = vault_container
                .go_to_parent_group()
                .expect("Should not fail.");
            assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
        });
    }

    #[test]
    fn edit_group() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut vault_container = VaultContainer::new(Box::new(vault));

        // Root should not be edible.
        assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
        assert!(vault_container.modify_group_view().is_none());
        assert_eq!(vault_container.current_view, CurrentView::ListView);
        let _ = vault_container
            .edit_group()
            .expect_err("Root should not be edible");
        assert!(vault_container.modify_group_view().is_none());
        assert_eq!(vault_container.current_view, CurrentView::ListView);

        // Set selected group to the third children of root
        let roots_3rd_children_uuid = vault_container
            .vault
            .get_groups_of(&root)
            .get(3)
            .unwrap()
            .uuid()
            .to_owned();
        let _ = vault_container
            .list_view
            .set_selected_group_uuid(roots_3rd_children_uuid.clone());

        // Non-root should be edible
        assert_eq!(
            vault_container.list_view.selected_group_uuid(),
            &roots_3rd_children_uuid
        );
        assert!(vault_container.modify_group_view().is_none());
        assert_eq!(vault_container.current_view, CurrentView::ListView);
        let _ = vault_container.edit_group().expect("Should not fail");
        assert!(vault_container.modify_group_view().is_some());
        assert_eq!(vault_container.current_view, CurrentView::ModifyGroup);

        let modify_group_view = vault_container.modify_group_view().as_ref().unwrap();
        assert_eq!(modify_group_view.group().uuid(), &roots_3rd_children_uuid);
    }

    #[test]
    fn select_group() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map.borrow_mut().insert(ListView::resize.type_id(), 0);

            ListView::resize.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ListView::resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            vault_container
                .list_view
                .set_search("This is some search string".into());

            assert!(!vault_container.list_view.search().is_empty());
            assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            let _ = vault_container.select_group([1; uuid::SIZE].into());
            assert!(vault_container.list_view.search().is_empty());
            assert_eq!(
                vault_container.list_view.selected_group_uuid(),
                &[1; uuid::SIZE].into()
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
        });
    }

    #[test]
    fn select_entry() {
        // From cache
        select_entry_parameterized(false);
        // From disk
        select_entry_parameterized(true);
    }

    fn select_entry_parameterized(from_disk: bool) {
        let mem_key = MemKey::with_length(1);
        let (_dir, mut vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        if from_disk {
            vault.save(&mem_key).unwrap();
        }

        let mut vault_container = VaultContainer::new(Box::new(vault));

        let roots_3rd_entry: EntryHead = vault_container
            .vault
            .get_entries_of(&root)
            .get(3)
            .map(|entry| (*entry).clone())
            .unwrap();

        let mutex_mem_key = Mutex::new(mem_key);

        assert!(vault_container.modify_group_view().is_none());
        assert_eq!(vault_container.current_view, CurrentView::ListView);
        let _ = vault_container
            .select_entry(roots_3rd_entry.uuid(), &mutex_mem_key.lock().unwrap())
            .expect("Should not fail");
        assert!(vault_container.modify_entry_view().is_some());
        assert_eq!(vault_container.current_view, CurrentView::ModifyEntry);

        let modify_entry_view = vault_container.modify_entry_view().as_ref().unwrap();
        assert_eq!(
            modify_entry_view.entry_head().uuid(),
            roots_3rd_entry.uuid()
        );
        assert_eq!(
            modify_entry_view.entry_head().title(),
            roots_3rd_entry.title()
        );
        assert_eq!(
            modify_entry_view.entry_body().uuid(),
            roots_3rd_entry.body()
        );
    }

    #[test]
    fn split_resize() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);

        let mut vault_container = VaultContainer::new(Box::new(vault));

        assert_ne!(
            vault_container.list_view.split_state().divider_position(),
            Some(1337)
        );
        vault_container.split_resize(1337);
        assert_eq!(
            vault_container.list_view.split_state().divider_position(),
            Some(1337)
        );
    }

    #[test]
    fn update_group_tree() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map.borrow_mut().insert(GroupTree::update.type_id(), 0);
            call_map.borrow_mut().insert(ListView::resize.type_id(), 0);

            GroupTree::update.mock_raw(|_self, _message, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&GroupTree::update.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            ListView::resize.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ListView::resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            // Toggle expansion
            assert_eq!(call_map.borrow()[&GroupTree::update.type_id()], 0);
            let _ =
                vault_container.update_group_tree(GroupTreeMessage::ToggleExpansion(vec![1, 2, 3]));
            assert_eq!(call_map.borrow()[&GroupTree::update.type_id()], 1);

            // Group selected
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
            let _ = vault_container
                .update_group_tree(GroupTreeMessage::GroupSelected([1; uuid::SIZE].into()));
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
            assert_eq!(
                vault_container.list_view.selected_group_uuid(),
                &[1; uuid::SIZE].into()
            );

            assert!(call_map.borrow().values().all(|v| *v == 1));
        })
    }

    #[test]
    fn update_list_items() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultContainer::select_group.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::select_entry.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::copy_username.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::copy_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::auto_fill::<TestPlatform>.type_id(), 0);

            VaultContainer::select_group.mock_raw(|_self, _uuid| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::select_group.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultContainer::select_entry.mock_raw(|_self, _uuid, _mem_key| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::select_entry.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::copy_username.mock_raw(|_self, _uuid, _mem_key, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::copy_username.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::copy_password.mock_raw(|_self, _uuid, _mem_key, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::copy_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::auto_fill::<TestPlatform>.mock_raw(|_self, _uuid, _mem_key| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::auto_fill::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Select group
            assert_eq!(
                call_map.borrow()[&VaultContainer::select_group.type_id()],
                0
            );
            let _ = vault_container.update_list_items::<TestPlatform>(
                ListItemMessage::GroupSelected([1; uuid::SIZE].into()),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultContainer::select_group.type_id()],
                1
            );

            // Select entry
            assert_eq!(
                call_map.borrow()[&VaultContainer::select_entry.type_id()],
                0
            );
            let _ = vault_container
                .update_list_items::<TestPlatform>(
                    ListItemMessage::EntrySelected([1; uuid::SIZE].into()),
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::select_entry.type_id()],
                1
            );

            // Copy username
            assert_eq!(
                call_map.borrow()[&VaultContainer::copy_username.type_id()],
                0
            );
            let _ = vault_container
                .update_list_items::<TestPlatform>(
                    ListItemMessage::CopyUsername([1; uuid::SIZE].into()),
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::copy_username.type_id()],
                1
            );

            // Copy password
            assert_eq!(
                call_map.borrow()[&VaultContainer::copy_password.type_id()],
                0
            );
            let _ = vault_container
                .update_list_items::<TestPlatform>(
                    ListItemMessage::CopyPassword([1; uuid::SIZE].into()),
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::copy_password.type_id()],
                1
            );

            // Auto fill
            assert_eq!(
                call_map.borrow()[&VaultContainer::auto_fill::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_container
                .update_list_items::<TestPlatform>(
                    ListItemMessage::Autofill([1; uuid::SIZE].into()),
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::auto_fill::<TestPlatform>.type_id()],
                1
            );

            assert!(call_map.borrow().values().all(|v| *v == 1));
        });
    }

    #[test]
    fn update_list() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultContainer::update_search.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::go_to_parent_group.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::edit_group.type_id(), 0);
            call_map.borrow_mut().insert(
                VaultContainer::update_list_items::<TestPlatform>.type_id(),
                0,
            );
            call_map
                .borrow_mut()
                .insert(VaultContainer::split_resize.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::update_group_tree.type_id(), 0);

            VaultContainer::update_search.mock_raw(|_self, _search| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::update_search.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultContainer::go_to_parent_group.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::go_to_parent_group.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::edit_group.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::edit_group.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::update_list_items::<TestPlatform>.mock_raw(
                |_self, _message, _clipboard| {
                    call_map
                        .borrow_mut()
                        .get_mut(&VaultContainer::update_list_items::<TestPlatform>.type_id())
                        .map(|c| *c += 1);
                    MockResult::Return(Ok(Command::none()))
                },
            );
            VaultContainer::split_resize.mock_raw(|_self, _position| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::split_resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            VaultContainer::update_group_tree.mock_raw(|_self, _message| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::update_group_tree.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Update search
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_search.type_id()],
                0
            );
            let _ = vault_container.update_list::<TestPlatform>(
                ListMessage::SearchInput("This is a search string".into()),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_search.type_id()],
                1
            );

            // Go to parent group
            assert_eq!(
                call_map.borrow()[&VaultContainer::go_to_parent_group.type_id()],
                0
            );
            let _ = vault_container.update_list::<TestPlatform>(ListMessage::Back, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&VaultContainer::go_to_parent_group.type_id()],
                1
            );

            // Edit group
            assert_eq!(call_map.borrow()[&VaultContainer::edit_group.type_id()], 0);
            let _ =
                vault_container.update_list::<TestPlatform>(ListMessage::EditGroup, &mut clipboard);
            assert_eq!(call_map.borrow()[&VaultContainer::edit_group.type_id()], 1);

            // Update list items
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_list_items::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_container.update_list::<TestPlatform>(
                ListMessage::ListItemMessage(ListItemMessage::GroupSelected(
                    [1; uuid::SIZE].into(),
                )),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_list_items::<TestPlatform>.type_id()],
                1
            );

            // Split resize
            assert_eq!(
                call_map.borrow()[&VaultContainer::split_resize.type_id()],
                0
            );
            let _ = vault_container
                .update_list::<TestPlatform>(ListMessage::SplitResize(10), &mut clipboard);
            assert_eq!(
                call_map.borrow()[&VaultContainer::split_resize.type_id()],
                1
            );

            // Update group tree
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_group_tree.type_id()],
                0
            );
            let _ = vault_container.update_list::<TestPlatform>(
                ListMessage::GroupTreeMessage(GroupTreeMessage::GroupSelected(
                    [1; uuid::SIZE].into(),
                )),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_group_tree.type_id()],
                1
            );

            assert!(call_map.borrow().values().all(|v| *v == 1));
        })
    }

    #[test]
    fn update_modify_group() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyGroupView::update.type_id(), 0);
            call_map.borrow_mut().insert(ListView::resize.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(GroupTree::refresh.type_id(), 0);

            ModifyGroupView::update.mock_raw(|_self, _m, _v, _mod, _s, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyGroupView::update.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            ListView::resize.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ListView::resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });
            GroupTree::refresh.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&GroupTree::refresh.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            // Non existent ModifyGroupView should not be updated.
            assert!(vault_container.modify_group_view.is_none());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 0);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 0);
            let cmd = vault_container
                .update_modify_group(
                    &ModifyGroupMessage::TitleInput("title".into()),
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert!(cmd.futures().is_empty());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 0);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 0);

            let roots_3rd_child_uuid = vault_container
                .vault
                .get_groups_of(&root)
                .get(3)
                .unwrap()
                .uuid()
                .to_owned();
            let _ = vault_container
                .list_view
                .set_selected_group_uuid(roots_3rd_child_uuid);
            let _ = vault_container.edit_group().expect("Should not fail");

            // Existent ModifyGroupView should be updated.
            assert!(vault_container.modify_group_view.is_some());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 0);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 0);
            let _ = vault_container
                .update_modify_group(
                    &ModifyGroupMessage::TitleInput("title".into()),
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 1);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 0);

            // Cancel should go back to the list view.
            assert_eq!(vault_container.current_view, CurrentView::ModifyGroup);
            assert!(vault_container.modify_group_view.is_some());
            let _ = vault_container
                .update_modify_group(
                    &ModifyGroupMessage::Cancel,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(vault_container.current_view, CurrentView::ListView);
            assert!(vault_container.modify_group_view.is_none());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 2);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 1);

            // Submit should go back to the list view.
            let _ = vault_container.edit_group().expect("Should not fail");
            assert_eq!(vault_container.current_view, CurrentView::ModifyGroup);
            assert!(vault_container.modify_group_view.is_some());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 2);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 1);
            let _ = vault_container
                .update_modify_group(
                    &ModifyGroupMessage::Submit,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert!(vault_container.modify_group_view.is_none());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 3);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 2);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 2);

            // Submit deletion of group should go back to the list view.
            let _ = vault_container.edit_group().expect("Should not fail");
            assert_eq!(vault_container.current_view, CurrentView::ModifyGroup);
            assert!(vault_container.modify_group_view.is_some());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 3);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 2);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 2);
            let _ = vault_container
                .update_modify_group(
                    &ModifyGroupMessage::Modal(ModifyGroupModalMessage::SubmitDelete),
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert!(vault_container.modify_group_view.is_none());
            assert_eq!(call_map.borrow()[&ModifyGroupView::update.type_id()], 4);
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 3);
            assert_eq!(call_map.borrow()[&GroupTree::refresh.type_id()], 3);
        });
    }

    #[test]
    fn update_modify_entry() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update::<TestPlatform>.type_id(), 0);
            call_map.borrow_mut().insert(ListView::resize.type_id(), 0);

            ModifyEntryView::update::<TestPlatform>.mock_raw(|_self, _m, _v, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            ListView::resize.mock_raw(|_self, _vault| {
                call_map
                    .borrow_mut()
                    .get_mut(&ListView::resize.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(())
            });

            // Non existent ModifyEntryView should not be updated.
            assert!(vault_container.modify_entry_view.is_none());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                0
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            let cmd = vault_container
                .update_modify_entry::<TestPlatform>(
                    &ModifyEntryMessage::TitleInput("title".into()),
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert!(cmd.futures().is_empty());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                0
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);

            let mutex_mem_key = Mutex::new(mem_key);
            let roots_3rd_entry_uuid = vault_container
                .vault
                .get_entries_of(&root)
                .get(3)
                .unwrap()
                .uuid()
                .to_owned();
            let _ =
                vault_container.select_entry(&roots_3rd_entry_uuid, &mutex_mem_key.lock().unwrap());

            // Existent ModifyEntryView should be updated.
            assert!(vault_container.modify_entry_view.is_some());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                0
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);
            let _ = vault_container
                .update_modify_entry::<TestPlatform>(
                    &ModifyEntryMessage::TitleInput("title".into()),
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                1
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 0);

            // Cancel should go back to the list view.
            assert_eq!(vault_container.current_view, CurrentView::ModifyEntry);
            assert!(vault_container.modify_entry_view.is_some());
            let _ = vault_container
                .update_modify_entry::<TestPlatform>(
                    &ModifyEntryMessage::Cancel,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(vault_container.current_view, CurrentView::ListView);
            assert!(vault_container.modify_entry_view.is_none());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                2
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);

            // Submit should go back to the list view.
            let _ =
                vault_container.select_entry(&roots_3rd_entry_uuid, &mutex_mem_key.lock().unwrap());
            assert_eq!(vault_container.current_view, CurrentView::ModifyEntry);
            assert!(vault_container.modify_entry_view.is_some());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                2
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 1);
            let _ = vault_container
                .update_modify_entry::<TestPlatform>(
                    &ModifyEntryMessage::Submit,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(vault_container.current_view, CurrentView::ListView);
            assert!(vault_container.modify_entry_view.is_none());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                3
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 2);

            // Submit deletion of entry should go back to the list view
            let _ =
                vault_container.select_entry(&roots_3rd_entry_uuid, &mutex_mem_key.lock().unwrap());
            assert_eq!(vault_container.current_view, CurrentView::ModifyEntry);
            assert!(vault_container.modify_entry_view.is_some());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                3
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 2);
            let _ = vault_container
                .update_modify_entry::<TestPlatform>(
                    &ModifyEntryMessage::Modal(ModifyEntryModalMessage::SubmitDelete),
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(vault_container.current_view, CurrentView::ListView);
            assert!(vault_container.modify_entry_view.is_none());
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update::<TestPlatform>.type_id()],
                4
            );
            assert_eq!(call_map.borrow()[&ListView::resize.type_id()], 3);
        });
    }

    #[test]
    fn new_container() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let vault_container = VaultContainer::new(Box::new(vault.clone()));

        assert_eq!(vault_container.vault.get_root_uuid().unwrap(), root);
        assert_eq!(vault_container.current_view, CurrentView::ListView);
        assert_eq!(vault_container.list_view.selected_group_uuid(), &root);
        assert!(vault_container.modify_group_view.is_none());
        assert!(vault_container.modify_entry_view.is_none());
    }

    #[test]
    fn title() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);

        let mut vault_container = VaultContainer::new(Box::new(vault));

        assert!(vault_container.vault.contains_unsaved_changes());
        assert!(vault_container
            .title()
            .contains(vault_container.vault.get_name()));
        assert!(vault_container.title().contains("\u{2731}")); // Big star symbol

        let mutex_mem_key = Mutex::new(mem_key);

        let _ = vault_container
            .save(&mutex_mem_key.lock().unwrap())
            .unwrap();

        assert!(!vault_container.vault.contains_unsaved_changes());
        assert!(vault_container
            .title()
            .contains(vault_container.vault.get_name()));
        assert!(!vault_container.title().contains("\u{2731}")); // Big star symbol
    }

    #[test]
    fn update() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        let mut vault_container = VaultContainer::new(Box::new(vault));

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultContainer::update_toolbar::<TestPlatform>.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::update_list::<TestPlatform>.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(VaultContainer::update_modify_group.type_id(), 0);
            call_map.borrow_mut().insert(
                VaultContainer::update_modify_entry::<TestPlatform>.type_id(),
                0,
            );

            VaultContainer::update_toolbar::<TestPlatform>.mock_raw(
                |_self, _message, _clipboard| {
                    call_map
                        .borrow_mut()
                        .get_mut(&VaultContainer::update_toolbar::<TestPlatform>.type_id())
                        .map(|c| *c += 1);
                    MockResult::Return(Ok(Command::none()))
                },
            );
            VaultContainer::update_list::<TestPlatform>.mock_raw(|_self, _message, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::update_list::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::update_modify_group.mock_raw(|_self, _m, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::update_modify_group.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            VaultContainer::update_modify_entry::<TestPlatform>.mock_raw(|_self, _m, _mod, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultContainer::update_modify_entry::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Update toolbar
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_toolbar::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_container
                .update::<TestPlatform>(
                    VaultContainerMessage::ToolBar(ToolBarMessage::LockVault),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_toolbar::<TestPlatform>.type_id()],
                1
            );

            // Update list
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_list::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_container
                .update::<TestPlatform>(
                    VaultContainerMessage::List(ListMessage::Back),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_list::<TestPlatform>.type_id()],
                1
            );

            // Update modify group
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_modify_group.type_id()],
                0
            );
            let _ = vault_container
                .update::<TestPlatform>(
                    VaultContainerMessage::ModifyGroup(ModifyGroupMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_modify_group.type_id()],
                1
            );

            // Update modify entry
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_modify_entry::<TestPlatform>.type_id()],
                0
            );
            let _ = vault_container
                .update::<TestPlatform>(
                    VaultContainerMessage::ModifyEntry(ModifyEntryMessage::Submit),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            assert_eq!(
                call_map.borrow()[&VaultContainer::update_modify_entry::<TestPlatform>.type_id()],
                1
            );

            // Autotype ok
            let _ = vault_container
                .update::<TestPlatform>(
                    VaultContainerMessage::AutoTypeResult(Ok(())),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");

            // Autotype error
            let _ = vault_container
                .update::<TestPlatform>(
                    VaultContainerMessage::AutoTypeResult(Err(PWDuckGuiError::String("".into()))),
                    &mut application_settings,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");

            assert!(call_map.borrow().values().all(|v| *v == 1));
        })
    }
}
