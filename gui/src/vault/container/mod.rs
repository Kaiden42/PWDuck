//! TODO

use iced::{button, Column, Command, Container, Length, Row, Space, Text};
use pwduck_core::{EntryBody, EntryHead, Group, Vault};

mod list;
use list::{ListMessage, ListView};

mod modify_entry;
use modify_entry::{ModifyEntryMessage, ModifyEntryView};

mod modify_group;
use getset::Getters;
use modify_group::{CreateGroupMessage, CreateGroupView};

use crate::{
    utils::icon_button, Component, Platform, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING,
    DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING, DEFAULT_SPACE_HEIGHT,
};

/// TODO
#[derive(Debug, Getters)]
pub struct VaultContainer {
    /// TODO
    #[getset(get = "pub")]
    vault: Vault,
    /// TODO
    current_view: CurrentView,
    /// TODO
    list_view: ListView,
    /// TODO
    create_group_view: CreateGroupView,
    /// TODO
    modify_entry_view: Option<Box<ModifyEntryView>>,

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

            save_state: button::State::new(),
            new_group_state: button::State::new(),
            new_entry_state: button::State::new(),
            copy_username_state: button::State::new(),
            copy_password_state: button::State::new(),
            lock_vault_state: button::State::new(),
        }
    }

    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match message {
            VaultContainerMessage::Save => {
                // TODO: find a way to do this async
                let mem_key = crate::MEM_KEY.lock().unwrap();
                self.vault.save(&mem_key).unwrap();

                Command::none()
            }
            VaultContainerMessage::NewGroup => {
                self.current_view = CurrentView::CreateGroup;
                Command::none()
            }
            VaultContainerMessage::NewEntry => {
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

                self.modify_entry_view =
                    Some(Box::new(ModifyEntryView::with(entry_head, entry_body)));
                self.current_view = CurrentView::ModifyEntry;

                Command::none()
            }
            VaultContainerMessage::CopyUsername => todo!(),
            VaultContainerMessage::CopyPassword => todo!(),
            VaultContainerMessage::LockVault => unreachable!(),

            VaultContainerMessage::ListMessage(message) => match message {
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
                        .set_selected_group_uuid(group.parent().clone());
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
                        pwduck_core::Uuid::new(self.vault.path()),
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

                        self.current_view = CurrentView::ListView;
                        self.modify_entry_view = None;
                        self.list_view.resize(&self.vault);
                    }

                    Command::none()
                }
            },
        }
    }

    fn view<P: Platform + 'static>(&mut self) -> iced::Element<'_, Self::Message> {
        let vault_contains_unsaved_changes = self.vault.contains_unsaved_changes();

        let mut save = icon_button(&mut self.save_state, "I", "Save Vault");
        if vault_contains_unsaved_changes && self.modify_entry_view.is_none() {
            save = save.on_press(VaultContainerMessage::Save);
        }

        let mut new_group = icon_button(&mut self.new_group_state, "I", "New Group");
        let mut new_entry = icon_button(&mut self.new_entry_state, "I", "New Entry");
        if
        /*TODO self.create_group_view.is_none() &&*/
        self.modify_entry_view.is_none() {
            new_group = new_group.on_press(VaultContainerMessage::NewGroup);
            new_entry = new_entry.on_press(VaultContainerMessage::NewEntry);
        }

        let mut copy_username = icon_button(&mut self.copy_username_state, "I", "Copy Username");
        let mut copy_password = icon_button(&mut self.copy_password_state, "I", "Copy Password");
        if self.modify_entry_view.is_some() {
            copy_username = copy_username.on_press(VaultContainerMessage::CopyUsername);
            copy_password = copy_password.on_press(VaultContainerMessage::CopyUsername);
        }

        let mut lock_vault = icon_button(&mut self.lock_vault_state, "I", "Lock Vault");
        if !vault_contains_unsaved_changes {
            lock_vault = lock_vault.on_press(VaultContainerMessage::LockVault)
        }

        let toolbar = Row::with_children(vec![
            save.into(),
            new_group.into(),
            new_entry.into(),
            copy_username.into(),
            copy_password.into(),
            lock_vault.into(),
        ])
        .spacing(DEFAULT_ROW_SPACING)
        .width(Length::Fill);

        let body = match self.current_view {
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
        };

        Container::new(
            Column::new()
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_COLUMN_SPACING)
                .push(
                    Text::new(&format!("Vault: {}", self.vault.get_name()))
                        .size(DEFAULT_HEADER_SIZE),
                )
                .push(toolbar)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(body),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
