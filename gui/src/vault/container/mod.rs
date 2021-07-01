//! TODO

use iced::{Column, Command, Container, Length, Text};
use pwduck_core::{EntryBody, EntryHead, Group, Vault};

mod list;
use list::{ListMessage, ListView};

mod modify_entry;
pub use modify_entry::ModifyEntryMessage;
use modify_entry::ModifyEntryView;

mod modify_group;
use getset::Getters;
use modify_group::{ModifyGroupMessage, ModifyGroupView};

mod toolbar;
use toolbar::ToolBar;
pub use toolbar::ToolBarMessage;

use crate::{
    error::PWDuckGuiError, utils::default_vertical_space, Component, Platform, Viewport,
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_HEADER_SIZE,
};

use self::list::ListItemMessage;

/// The state of the vault container.
#[derive(Debug, Getters)]
pub struct VaultContainer {
    /// The unlocked vault.
    #[getset(get = "pub")]
    vault: Box<Vault>,

    /// The state of the [`ToolBar`](ToolBar).
    tool_bar: ToolBar,

    /// The state of the current view.
    current_view: CurrentView,

    /// The state of the list view.
    list_view: ListView,

    /// The state of the group modification view.
    modify_group_view: Option<Box<ModifyGroupView>>,

    /// The state of the entry modification view.
    modify_entry_view: Option<Box<ModifyEntryView>>,
}

impl VaultContainer {
    /// Save the vault to disk.
    fn save(&mut self) -> Result<Command<ToolBarMessage>, PWDuckGuiError> {
        // TODO: find a way to do this async
        let mem_key = crate::MEM_KEY.lock()?;
        self.vault.save(&mem_key)?;
        Ok(Command::none())
    }

    /// Create a new group and switch to the [`ModifyGroupView`](ModifyGroupView) as the current view.
    fn create_group(&mut self) -> Command<ToolBarMessage> {
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
    fn create_entry(&mut self) -> Command<ToolBarMessage> {
        let entry_body = EntryBody::new(
            pwduck_core::Uuid::new(self.vault.path()),
            String::new(),
            String::new(),
        );
        let entry_head = EntryHead::new(
            pwduck_core::Uuid::new(self.vault.path()),
            self.list_view.selected_group_uuid().clone(),
            String::new(),
            entry_body.uuid().as_string(),
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
    fn copy_username(
        &self,
        uuid: &str,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let mem_key = crate::MEM_KEY.lock()?;
        let masterkey = self.vault.masterkey().as_unprotected(
            &mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        let entry_body = self.vault.unsaved_entry_bodies().get(uuid).map_or_else(
            || pwduck_core::EntryBody::load(self.vault.path(), uuid, &masterkey),
            |dto| pwduck_core::EntryBody::decrypt(dto, &masterkey),
        )?;

        clipboard.write(entry_body.username().clone());

        Ok(Command::none())
    }

    /// Copy the password to the clipboard.
    fn copy_password(
        &self,
        uuid: &str,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let mem_key = crate::MEM_KEY.lock()?;
        let masterkey = self.vault.masterkey().as_unprotected(
            &mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        let entry_body = self.vault.unsaved_entry_bodies().get(uuid).map_or_else(
            || pwduck_core::EntryBody::load(self.vault.path(), uuid, &masterkey),
            |dto| pwduck_core::EntryBody::decrypt(dto, &masterkey),
        )?;

        clipboard.write(entry_body.password().clone());

        Ok(Command::none())
    }

    /// Update the [`ToolBar`](ToolBar) with the given message.
    fn update_toolbar(
        &mut self,
        message: &ToolBarMessage,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ToolBarMessage::Save => self.save(),
            ToolBarMessage::NewGroup => Ok(self.create_group()),
            ToolBarMessage::NewEntry => Ok(self.create_entry()),
            ToolBarMessage::AutoFill => todo!(),
            ToolBarMessage::LockVault => {
                PWDuckGuiError::Unreachable("ToolBarMessage".into()).into()
            }
        }
        .map(|cmd| cmd.map(VaultContainerMessage::ToolBar))
    }

    /// Update the search and replace it with the given value. The [`ListView`](ListView) will be resized.
    fn update_search(&mut self, search: String) -> Command<VaultContainerMessage> {
        self.list_view.set_search(search);
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
        self.list_view
            .set_selected_group_uuid(group.parent().clone());
        self.list_view.resize(&self.vault);
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
        self.modify_group_view = Some(Box::new(ModifyGroupView::with(
            modify_group::State::Modify,
            group,
        )));
        self.current_view = CurrentView::ModifyGroup;
        Ok(Command::none())
    }

    /// Select the group identified by the UUID.
    fn select_group(&mut self, uuid: String) -> Command<VaultContainerMessage> {
        self.list_view.set_selected_group_uuid(uuid);
        self.list_view.search_mut().clear();
        self.list_view.resize(&self.vault);
        Command::none()
    }

    /// Select the entry identified by the UUID. It will be loaded, decrypted
    /// and finally displayed in the [`ModifyEntryView`](ModifyEntryView).
    fn select_entry(
        &mut self,
        uuid: &str,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let entry_head = self
            .vault
            .entries()
            .get(uuid)
            .ok_or(PWDuckGuiError::Option)?
            .clone();

        let mem_key = crate::MEM_KEY.lock()?;
        let masterkey = self.vault.masterkey().as_unprotected(
            &mem_key,
            self.vault.salt(),
            self.vault.nonce(),
        )?;

        // Load body from cache if exists, otherwise load from file system.
        let entry_body = self
            .vault
            .unsaved_entry_bodies()
            .get(entry_head.body())
            .map_or_else(
                || pwduck_core::EntryBody::load(self.vault.path(), entry_head.body(), &masterkey),
                |dto| pwduck_core::EntryBody::decrypt(dto, &masterkey),
            )?;

        self.modify_entry_view = Some(Box::new(ModifyEntryView::with(
            modify_entry::State::Modify,
            entry_head,
            entry_body,
        )));
        self.current_view = CurrentView::ModifyEntry;
        Ok(Command::none())
    }

    /// Handle the message that was send by the list items.
    fn update_list_items(
        &mut self,
        message: ListItemMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ListItemMessage::GroupSelected(uuid) => Ok(self.select_group(uuid)),
            ListItemMessage::EntrySelected(uuid) => self.select_entry(&uuid),
            ListItemMessage::CopyUsername(uuid) => self.copy_username(&uuid, clipboard),
            ListItemMessage::CopyPassword(uuid) => self.copy_password(&uuid, clipboard),
            ListItemMessage::Autofill(_) => todo!(),
        }
    }

    /// Handle the message that was send by the [`ListView`](ListView).
    fn update_list(
        &mut self,
        message: ListMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ListMessage::SearchInput(search) => Ok(self.update_search(search)),
            ListMessage::Back => self.go_to_parent_group(),
            ListMessage::EditGroup => self.edit_group(),
            ListMessage::ListItemMessage(message) => self.update_list_items(message, clipboard),
            ListMessage::SplitResize(position) => {
                self.list_view
                    .split_state_mut()
                    .set_divider_position(position);
                Ok(Command::none())
            }
            ListMessage::GroupTreeMessage(message) => match message {
                list::GroupTreeMessage::ToggleExpansion(_) => self
                    .list_view
                    .group_tree_mut()
                    .update(message, &self.vault)
                    .map(|cmd| {
                        cmd.map(|msg| {
                            VaultContainerMessage::List(ListMessage::GroupTreeMessage(msg))
                        })
                    }),
                list::GroupTreeMessage::GroupSelected(uuid) => {
                    self.list_view.set_selected_group_uuid(uuid);
                    self.list_view.resize(&self.vault);
                    Ok(Command::none())
                }
            },
        }
    }

    /// Handle the massage that was send by the [`ModifyGroupView`](ModifyGroupView).
    fn update_modify_group(
        &mut self,
        message: ModifyGroupMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ModifyGroupMessage::Cancel => {
                self.current_view = CurrentView::ListView;
                self.modify_group_view = None;
                Ok(Command::none())
            }
            ModifyGroupMessage::Submit => {
                if let Some(modify_group_view) = self.modify_group_view.as_mut() {
                    self.vault.insert_group(modify_group_view.group().clone());

                    self.list_view.resize(&self.vault);
                    self.list_view.group_tree_mut().refresh(&self.vault);
                    self.current_view = CurrentView::ListView;
                    self.modify_group_view = None
                }
                Ok(Command::none())
            }
            _ => self
                .modify_group_view
                .as_mut()
                .map_or_else(
                    || Ok(Command::none()),
                    |view| view.update(message, clipboard),
                )
                .map(|cmd| cmd.map(VaultContainerMessage::ModifyGroup)),
        }
    }

    /// Handle the message that was send by the [`ModifyEntryView`](ModifyEntryView).
    fn update_modify_entry(
        &mut self,
        message: ModifyEntryMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ModifyEntryMessage::Cancel => {
                self.current_view = CurrentView::ListView;
                self.modify_entry_view = None;
                Ok(Command::none())
            }

            ModifyEntryMessage::Submit => {
                if let Some(modify_entry_view) = self.modify_entry_view.as_ref() {
                    // TODO async
                    let mem_key = crate::MEM_KEY.lock()?;
                    let masterkey = self.vault.masterkey().as_unprotected(
                        &mem_key,
                        self.vault.salt(),
                        self.vault.nonce(),
                    )?;

                    self.vault.insert_entry(
                        modify_entry_view.entry_head().clone(),
                        modify_entry_view.entry_body().clone(),
                        &masterkey,
                    )?;

                    self.current_view = CurrentView::ListView;
                    self.modify_entry_view = None;
                    self.list_view.resize(&self.vault);
                }

                Ok(Command::none())
            }

            _ => self
                .modify_entry_view
                .as_mut()
                .map_or_else(
                    || Ok(Command::none()),
                    |view| view.update(message, clipboard),
                )
                .map(|cmd| cmd.map(VaultContainerMessage::ModifyEntry)),
        }
    }
}

/// The current view to display.
#[derive(Debug)]
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
}

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

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<Self::Message>, PWDuckGuiError> {
        match message {
            VaultContainerMessage::ToolBar(message) => self.update_toolbar(&message, clipboard),

            VaultContainerMessage::List(message) => self.update_list(message, clipboard),

            VaultContainerMessage::ModifyGroup(message) => {
                self.update_modify_group(message, clipboard)
            }

            VaultContainerMessage::ModifyEntry(message) => {
                self.update_modify_entry(message, clipboard)
            }
        }
    }

    fn view<P: Platform + 'static>(
        &mut self,
        viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message> {
        let vault_contains_unsaved_changes = self.vault.contains_unsaved_changes();

        let hide_toolbar_labels = viewport.width < 800;

        let tool_bar = self
            .tool_bar
            .view(
                vault_contains_unsaved_changes,
                self.modify_entry_view.is_some(),
                self.modify_group_view.is_some(),
                hide_toolbar_labels,
            )
            .map(VaultContainerMessage::ToolBar);

        let body = match self.current_view {
            CurrentView::ListView => self
                .list_view
                .view(&self.vault, viewport)
                .map(VaultContainerMessage::List),

            CurrentView::ModifyGroup => match &mut self.modify_group_view {
                Some(modify_group_view) => modify_group_view
                    .view(&self.vault, self.list_view.selected_group_uuid())
                    .map(VaultContainerMessage::ModifyGroup),
                None => unreachable!(),
            },

            CurrentView::ModifyEntry => match &mut self.modify_entry_view {
                Some(modify_enty_view) => modify_enty_view
                    .view(self.list_view.selected_group_uuid())
                    .map(VaultContainerMessage::ModifyEntry),
                None => unreachable!(),
            },
        };

        Container::new(
            Column::new()
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_COLUMN_SPACING)
                .push(
                    Text::new(&format!("Vault: {}", self.vault.get_name()))
                        .size(DEFAULT_HEADER_SIZE),
                )
                .push(tool_bar)
                .push(default_vertical_space())
                .push(body),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
