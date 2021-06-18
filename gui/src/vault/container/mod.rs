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
    error::PWDuckGuiError, utils::default_vertical_space, Component, Platform,
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_HEADER_SIZE,
};

use self::list::ListItemMessage;

/// TODO
#[derive(Debug, Getters)]
pub struct VaultContainer {
    /// TODO
    #[getset(get = "pub")]
    vault: Box<Vault>,

    /// TODO
    tool_bar: ToolBar,

    /// TODO
    current_view: CurrentView,

    /// TODO
    list_view: ListView,

    /// TODO
    modify_group_view: Option<Box<ModifyGroupView>>,

    /// TODO
    modify_entry_view: Option<Box<ModifyEntryView>>,
}

impl VaultContainer {
    /// TODO
    fn save(&mut self) -> Result<Command<ToolBarMessage>, PWDuckGuiError> {
        // TODO: find a way to do this async
        let mem_key = crate::MEM_KEY.lock()?;
        self.vault.save(&mem_key)?;
        Ok(Command::none())
    }

    /// TODO
    fn create_group(&mut self) -> Command<ToolBarMessage> {
        let group = Group::new(
            pwduck_core::Uuid::new(self.vault.path()),
            self.list_view.selected_group_uuid().clone(),
            String::new(),
        );

        self.modify_group_view = Some(Box::new(ModifyGroupView::with(group)));
        self.current_view = CurrentView::CreateGroup;
        Command::none()
    }

    /// TODO
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

        self.modify_entry_view = Some(Box::new(ModifyEntryView::with(entry_head, entry_body)));
        self.current_view = CurrentView::ModifyEntry;

        Command::none()
    }

    /// TODO
    fn copy_username(&mut self, clipboard: &mut iced::Clipboard) -> Command<ToolBarMessage> {
        if let Some(modify_entry_view) = self.modify_entry_view.as_ref() {
            clipboard.write(modify_entry_view.entry_body().username().clone());
        }

        Command::none()
    }

    /// TODO
    fn copy_password(&mut self, clipboard: &mut iced::Clipboard) -> Command<ToolBarMessage> {
        if let Some(modify_entry_view) = self.modify_entry_view.as_ref() {
            clipboard.write(modify_entry_view.entry_body().password().clone());
        }

        Command::none()
    }

    /// TODO
    fn update_toolbar(
        &mut self,
        message: &ToolBarMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ToolBarMessage::Save => self.save(),
            ToolBarMessage::NewGroup => Ok(self.create_group()),
            ToolBarMessage::NewEntry => Ok(self.create_entry()),
            ToolBarMessage::CopyUsername => Ok(self.copy_username(clipboard)),
            ToolBarMessage::CopyPassword => Ok(self.copy_password(clipboard)),
            ToolBarMessage::LockVault => {
                PWDuckGuiError::Unreachable("ToolBarMessage".into()).into()
            }
        }
        .map(|cmd| cmd.map(VaultContainerMessage::ToolBar))
    }

    /// TODO
    fn update_search(&mut self, search: String) -> Command<VaultContainerMessage> {
        self.list_view.set_search(search);
        self.list_view.resize(&self.vault);
        Command::none()
    }

    /// TODO
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

    /// TODO
    fn edit_group(&mut self) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        let group = self
            .vault
            .groups()
            .get(self.list_view.selected_group_uuid())
            .ok_or(PWDuckGuiError::Option)?
            .clone();
        self.modify_group_view = Some(Box::new(ModifyGroupView::with(group)));
        self.current_view = CurrentView::CreateGroup;
        Ok(Command::none())
    }

    /// TODO
    fn select_group(&mut self, uuid: String) -> Command<VaultContainerMessage> {
        self.list_view.set_selected_group_uuid(uuid);
        self.list_view.resize(&self.vault);
        Command::none()
    }

    /// TODO
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

        self.modify_entry_view = Some(Box::new(ModifyEntryView::with(entry_head, entry_body)));
        self.current_view = CurrentView::ModifyEntry;
        Ok(Command::none())
    }

    /// TODO
    fn update_list_items(
        &mut self,
        message: ListItemMessage,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ListItemMessage::GroupSelected(uuid) => Ok(self.select_group(uuid)),
            ListItemMessage::EntrySelected(uuid) => self.select_entry(&uuid),
        }
    }

    /// TODO
    fn update_list(
        &mut self,
        message: ListMessage,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
        match message {
            ListMessage::SearchInput(search) => Ok(self.update_search(search)),
            ListMessage::Back => self.go_to_parent_group(),
            ListMessage::EditGroup => self.edit_group(),
            ListMessage::ListItemMessage(message) => self.update_list_items(message),
        }
    }

    /// TODO
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
                .map(|cmd| cmd.map(VaultContainerMessage::CreateGroup)),
        }
    }

    /// TODO
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

/// TODO
#[derive(Debug)]
enum CurrentView {
    /// TODO
    ListView,
    /// TODO
    CreateGroup,
    /// TODO
    ModifyEntry,
}

/// TODO
#[derive(Clone, Debug)]
pub enum VaultContainerMessage {
    /// TODO
    ToolBar(ToolBarMessage),
    /// TODO
    List(ListMessage),
    /// TODO
    CreateGroup(ModifyGroupMessage),
    /// TODO
    ModifyEntry(ModifyEntryMessage),
}

impl Component for VaultContainer {
    type Message = VaultContainerMessage;
    type ConstructorParam = Box<Vault>;

    fn new(vault: Self::ConstructorParam) -> Self {
        let root_uuid = vault.get_root_uuid().unwrap();
        let (group_count, entry_count) = (
            vault.get_groups_of(&root_uuid).len(),
            vault.get_entries_of(&root_uuid).len(),
        );
        Self {
            vault,
            tool_bar: ToolBar::default(),
            current_view: CurrentView::ListView,
            list_view: ListView::new(root_uuid, group_count, entry_count),
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

            VaultContainerMessage::CreateGroup(message) => {
                self.update_modify_group(message, clipboard)
            }

            VaultContainerMessage::ModifyEntry(message) => {
                self.update_modify_entry(message, clipboard)
            }
        }
    }

    fn view<P: Platform + 'static>(&mut self) -> iced::Element<'_, Self::Message> {
        let vault_contains_unsaved_changes = self.vault.contains_unsaved_changes();

        let tool_bar = self
            .tool_bar
            .view(
                vault_contains_unsaved_changes,
                self.modify_entry_view.is_some(),
                self.modify_group_view.is_some(),
            )
            .map(VaultContainerMessage::ToolBar);

        let body = match self.current_view {
            CurrentView::ListView => self
                .list_view
                .view(&self.vault)
                .map(VaultContainerMessage::List),

            CurrentView::CreateGroup => match &mut self.modify_group_view {
                Some(modify_group_view) => modify_group_view
                    .view(&self.vault, self.list_view.selected_group_uuid())
                    .map(VaultContainerMessage::CreateGroup),
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
