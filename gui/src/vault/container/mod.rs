//! TODO

use iced::Command;
use pwduck_core::{EntryBody, EntryHead, Group, PWDuckCoreError, Vault};

mod list;
use list::{ListMessage, ListView};

mod modify_entry;
use modify_entry::{ModifyEntryMessage, ModifyEntryView};

mod modify_group;
use modify_group::{CreateGroupMessage, CreateGroupView};

use crate::{Component, Platform};

/// TODO
#[derive(Debug)]
pub struct VaultContainer {
    vault: Vault,
    current_view: CurrentView,
    list_view: ListView,
    create_group_view: CreateGroupView,
    modify_entry_view: Option<Box<ModifyEntryView>>,
}

#[derive(Debug)]
enum CurrentView {
    ListView,
    CreateGroup,
    ModifyEntry,
}

/// TODO
#[derive(Clone, Debug)]
pub enum VaultContainerMessage {
    /// TODO
    ListMessage(ListMessage),
    /// TODO
    CreateGroupMessage(CreateGroupMessage),
    /// TODO
    ModifyEntryMessage(ModifyEntryMessage),
}

impl Component for VaultContainer {
    type Message = VaultContainerMessage;
    type ConstructorParam = Vault;

    fn new(vault: Self::ConstructorParam) -> Self {
        let root_uuid = vault.get_root_uuid().unwrap();
        let (group_count, entry_count) = (
            vault.get_groups_of(&root_uuid).len(),
            vault.get_entries_of(&root_uuid).len(),
        );
        Self {
            vault,
            current_view: CurrentView::ListView,
            list_view: ListView::new(root_uuid, group_count, entry_count),
            create_group_view: CreateGroupView::new(),
            modify_entry_view: None,
        }
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match message {
            VaultContainerMessage::ListMessage(message) => match message {
                ListMessage::Save => {
                    // TODO: find a way to do this async
                    let mem_key = crate::MEM_KEY.lock().unwrap();
                    self.vault.save(&mem_key).unwrap();

                    Command::none()
                }
                ListMessage::NewGroup => {
                    self.current_view = CurrentView::CreateGroup;
                    Command::none()
                }
                ListMessage::NewEntry => {
                    let entry_body = EntryBody::new(
                        pwduck_core::Uuid::new(&self.vault.path()),
                        String::new(),
                        String::new(),
                    );
                    let entry_head = EntryHead::new(
                        pwduck_core::Uuid::new(&self.vault.path()),
                        self.list_view.selected_group_uuid().to_owned(),
                        String::new(),
                        entry_body.uuid().as_string(),
                    );

                    self.modify_entry_view =
                        Some(Box::new(ModifyEntryView::with(entry_head, entry_body)));
                    self.current_view = CurrentView::ModifyEntry;

                    Command::none()
                }
                ListMessage::CopyUsername => Command::none(),
                ListMessage::CopyPassword => Command::none(),
                ListMessage::LockVault => unreachable!(),
                ListMessage::SearchInput(input) => {
                    //self.list_view.search = input;
                    self.list_view.set_search(input);
                    self.list_view.resize(&self.vault);
                    Command::none()
                }
                ListMessage::Back => {
                    let group = self
                        .vault
                        .groups()
                        .get(self.list_view.selected_group_uuid())
                        .unwrap();
                    self.list_view
                        .set_selected_group_uuid(group.parent().to_owned());
                    Command::none()
                }
                ListMessage::ListItemMessage(msg) => match msg {
                    list::ListItemMessage::GroupSelected(uuid) => {
                        self.list_view.set_selected_group_uuid(uuid);
                        Command::none()
                    }
                    list::ListItemMessage::EntrySelected(_uuid) => todo!(),
                },
            },
            VaultContainerMessage::CreateGroupMessage(message) => match message {
                CreateGroupMessage::GroupNameInput(input) => {
                    //self.create_group_view.group_name = input;
                    self.create_group_view.set_group_name(input);
                    Command::none()
                }
                CreateGroupMessage::Cancel => {
                    self.current_view = CurrentView::ListView;
                    Command::none()
                }
                CreateGroupMessage::Submit => {
                    let group = Group::new(
                        pwduck_core::Uuid::new(&self.vault.path()),
                        //self.list_view.selected_group_uuid.clone(),
                        self.list_view.selected_group_uuid().clone(),
                        //self.create_group_view.group_name.clone(),
                        self.create_group_view.group_name().clone(),
                    );

                    self.vault.add_group(group);

                    self.list_view.resize(&self.vault);
                    self.current_view = CurrentView::ListView;
                    self.create_group_view = CreateGroupView::new();
                    Command::none()
                }
            },

            VaultContainerMessage::ModifyEntryMessage(message) => match message {
                ModifyEntryMessage::TitleInput(input) => {
                    if let Some(modify_entry_view) = self.modify_entry_view.as_mut() {
                        modify_entry_view.entry_head_mut().set_title(input);
                    }

                    Command::none()
                }
                ModifyEntryMessage::UsernameInput(input) => {
                    if let Some(modify_entry_view) = self.modify_entry_view.as_mut() {
                        modify_entry_view.entry_body_mut().set_username(input);
                    }

                    Command::none()
                }
                ModifyEntryMessage::PasswordInput(input) => {
                    if let Some(modify_entry_view) = self.modify_entry_view.as_mut() {
                        modify_entry_view.entry_body_mut().set_password(input);
                    }

                    Command::none()
                }
                ModifyEntryMessage::Cancel => {
                    self.current_view = CurrentView::ListView;
                    self.modify_entry_view = None;

                    Command::none()
                }
                ModifyEntryMessage::Submit => {
                    if let Some(modify_entry_view) = self.modify_entry_view.as_mut() {
                        let mem_key = crate::MEM_KEY.lock().unwrap();
                        let masterkey = self
                            .vault
                            .masterkey()
                            .as_unprotected(&mem_key, self.vault.salt(), self.vault.nonce())
                            .unwrap();

                        self.vault
                            .add_entry(
                                modify_entry_view.entry_head().clone(),
                                modify_entry_view.entry_body().clone(),
                                &masterkey,
                            )
                            .unwrap();
                    }

                    Command::none()
                }
            },
        }
    }

    fn view<P: Platform + 'static>(&mut self) -> iced::Element<'_, Self::Message> {
        match self.current_view {
            CurrentView::ListView => self
                .list_view
                .view(&self.vault)
                .map(VaultContainerMessage::ListMessage),
            CurrentView::CreateGroup => self
                .create_group_view
                .view(&self.vault, self.list_view.selected_group_uuid())
                .map(VaultContainerMessage::CreateGroupMessage),
            CurrentView::ModifyEntry => match &mut self.modify_entry_view {
                Some(modify_enty_view) => modify_enty_view
                    .view(self.list_view.selected_group_uuid())
                    .map(VaultContainerMessage::ModifyEntryMessage),
                None => unreachable!(),
            },
        }
    }
}
