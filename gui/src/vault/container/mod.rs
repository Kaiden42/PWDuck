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
            VaultContainerMessage::ToolBar(message) => update_toolbar(self, &message, clipboard),
            VaultContainerMessage::List(message) => update_list(self, message, clipboard),
            VaultContainerMessage::CreateGroup(message) => {
                update_modify_group(self, message, clipboard)
            }
            VaultContainerMessage::ModifyEntry(message) => {
                update_modify_entry(self, message, clipboard)
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

/// TODO
fn update_toolbar(
    container: &mut VaultContainer,
    message: &ToolBarMessage,
    clipboard: &mut iced::Clipboard,
) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
    let cmd = match message {
        ToolBarMessage::Save => {
            // TODO: find a way to do this async
            let mem_key = crate::MEM_KEY.lock().unwrap();
            container.vault.save(&mem_key).unwrap();

            Command::none()
        }
        ToolBarMessage::NewGroup => {
            let group = Group::new(
                pwduck_core::Uuid::new(container.vault.path()),
                container.list_view.selected_group_uuid().clone(),
                String::new(),
            );

            container.modify_group_view = Some(Box::new(ModifyGroupView::with(group)));
            container.current_view = CurrentView::CreateGroup;
            Command::none()
        }
        ToolBarMessage::NewEntry => {
            let entry_body = EntryBody::new(
                pwduck_core::Uuid::new(container.vault.path()),
                String::new(),
                String::new(),
            );
            let entry_head = EntryHead::new(
                pwduck_core::Uuid::new(container.vault.path()),
                container.list_view.selected_group_uuid().clone(),
                String::new(),
                entry_body.uuid().as_string(),
            );

            container.modify_entry_view =
                Some(Box::new(ModifyEntryView::with(entry_head, entry_body)));
            container.current_view = CurrentView::ModifyEntry;

            Command::none()
        }
        ToolBarMessage::CopyUsername => {
            if let Some(modify_entry_view) = container.modify_entry_view.as_ref() {
                clipboard.write(modify_entry_view.entry_body().username().clone());
            }

            Command::none()
        }
        ToolBarMessage::CopyPassword => {
            if let Some(modify_entry_view) = container.modify_entry_view.as_ref() {
                clipboard.write(modify_entry_view.entry_body().password().clone());
            }

            Command::none()
        }
        ToolBarMessage::LockVault => {
            return PWDuckGuiError::Unreachable("ToolBarMessage".into()).into()
        }
    };
    Ok(cmd)
}

/// TODO
fn update_list(
    container: &mut VaultContainer,
    message: ListMessage,
    _clipboard: &mut iced::Clipboard,
) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
    let cmd = match message {
        ListMessage::SearchInput(input) => {
            //self.list_view.search = input;
            container.list_view.set_search(input);
            container.list_view.resize(&container.vault);
            Command::none()
        }
        ListMessage::Back => {
            let group = container
                .vault
                .groups()
                .get(container.list_view.selected_group_uuid())
                .unwrap();
            container
                .list_view
                .set_selected_group_uuid(group.parent().clone());
            container.list_view.resize(&container.vault);
            Command::none()
        }
        ListMessage::EditGroup => {
            let group = container
                .vault
                .groups()
                .get(container.list_view.selected_group_uuid())
                .ok_or(PWDuckGuiError::Option)?
                .clone();

            container.modify_group_view = Some(Box::new(ModifyGroupView::with(group)));
            container.current_view = CurrentView::CreateGroup;
            Command::none()
        }
        ListMessage::ListItemMessage(msg) => match msg {
            list::ListItemMessage::GroupSelected(uuid) => {
                container.list_view.set_selected_group_uuid(uuid);
                container.list_view.resize(&container.vault);
                Command::none()
            }
            list::ListItemMessage::EntrySelected(uuid) => {
                let entry_head = container
                    .vault
                    .entries()
                    .get(&uuid)
                    .ok_or(PWDuckGuiError::Option)?
                    .clone();

                let mem_key = crate::MEM_KEY.lock()?;
                let masterkey = container.vault.masterkey().as_unprotected(
                    &mem_key,
                    container.vault.salt(),
                    container.vault.nonce(),
                )?;

                // Load body from cache if exists, otherwise load from file system.
                let entry_body = container
                    .vault
                    .unsaved_entry_bodies()
                    .get(entry_head.body())
                    .map_or_else(
                        || {
                            pwduck_core::EntryBody::load(
                                container.vault.path(),
                                entry_head.body(),
                                &masterkey,
                            )
                        },
                        |dto| pwduck_core::EntryBody::decrypt(dto, &masterkey),
                    )?;

                container.modify_entry_view =
                    Some(Box::new(ModifyEntryView::with(entry_head, entry_body)));
                container.current_view = CurrentView::ModifyEntry;
                Command::none()
            }
        },
    };
    Ok(cmd)
}

/// TODO
fn update_modify_group(
    container: &mut VaultContainer,
    message: ModifyGroupMessage,
    _clipboard: &mut iced::Clipboard,
) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
    let cmd = match message {
        ModifyGroupMessage::GroupNameInput(input) => {
            if let Some(modify_group_view) = container.modify_group_view.as_mut() {
                //self.create_group_view.group_name = input;
                //modify_group_view.set_group_name(input);
                modify_group_view.group_mut().set_title(input);
            }
            Command::none()
        }
        ModifyGroupMessage::Cancel => {
            container.current_view = CurrentView::ListView;
            container.modify_group_view = None;

            Command::none()
        }
        ModifyGroupMessage::Submit => {
            if let Some(modify_group_view) = container.modify_group_view.as_mut() {
                container
                    .vault
                    .insert_group(modify_group_view.group().clone());

                container.list_view.resize(&container.vault);
                container.current_view = CurrentView::ListView;
                container.modify_group_view = None
            }
            Command::none()
        }
    };
    Ok(cmd)
}

/// TODO
fn update_modify_entry(
    container: &mut VaultContainer,
    message: ModifyEntryMessage,
    clipboard: &mut iced::Clipboard,
) -> Result<Command<VaultContainerMessage>, PWDuckGuiError> {
    let cmd = if let Some(modify_entry_view) = container.modify_entry_view.as_mut() {
        match message {
            ModifyEntryMessage::TitleInput(input) => {
                modify_entry_view.entry_head_mut().set_title(input);
                Command::none()
            }
            ModifyEntryMessage::UsernameInput(input) => {
                modify_entry_view.entry_body_mut().set_username(input);
                Command::none()
            }
            ModifyEntryMessage::UsernameCopy => {
                clipboard.write(modify_entry_view.entry_body().username().clone());
                Command::none()
            }
            ModifyEntryMessage::PasswordInput(input) => {
                modify_entry_view.entry_body_mut().set_password(input);
                Command::none()
            }
            ModifyEntryMessage::PasswordShow => {
                modify_entry_view.set_password_show(!modify_entry_view.password_show());
                Command::none()
            }
            ModifyEntryMessage::PasswordGenerate => {
                // TODO
                Command::none()
            }
            ModifyEntryMessage::PasswordCopy => {
                clipboard.write(modify_entry_view.entry_body().password().clone());
                Command::none()
            }
            ModifyEntryMessage::Cancel => {
                container.current_view = CurrentView::ListView;
                container.modify_entry_view = None;

                Command::none()
            }
            ModifyEntryMessage::Submit => {
                // TODO async
                let mem_key = crate::MEM_KEY.lock()?;
                let masterkey = container
                    .vault
                    .masterkey()
                    .as_unprotected(&mem_key, container.vault.salt(), container.vault.nonce())
                    .unwrap();

                container
                    .vault
                    .insert_entry(
                        modify_entry_view.entry_head().clone(),
                        modify_entry_view.entry_body().clone(),
                        &masterkey,
                    )
                    .unwrap();

                container.current_view = CurrentView::ListView;
                container.modify_entry_view = None;
                container.list_view.resize(&container.vault);

                Command::none()
            }
        }
    } else {
        Command::none()
    };
    Ok(cmd)
}
