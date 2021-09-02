//! TODO
use std::mem::swap;

use iced::{
    button, scrollable, text_input, Button, Column, Command, Container, Element, Length, Row,
    Scrollable, Space, Text, TextInput, VerticalAlignment,
};
use iced_aw::{split, Split};
use iced_focus::Focus;
use pwduck_core::{EntryHead, Group, Uuid, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::{Icon, ICON_FONT},
    theme::Theme,
    utils::{
        default_vertical_space, icon_button, icon_button_with_width, icon_text, vertical_space,
        ButtonData, ButtonKind, SomeIf,
    },
    Viewport, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};
use getset::{Getters, MutGetters, Setters};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the list view inside the vault container.
///
/// See: [`VaultContainer`](crate::vault::container::VaultContainer)
#[derive(Debug, Getters, MutGetters, Setters, Focus)]
pub struct ListView {
    /// The UUID of the selected group.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    selected_group_uuid: Uuid,
    /// The sub-groups of the selected group.
    #[getset(get)]
    #[focus(enable)]
    group_items: Vec<ListGroupItem>,
    /// The entries of the selected group.
    #[getset(get)]
    #[focus(enable)]
    entry_items: Vec<ListEntryItem>,

    /// The search string to search for groups / entries.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    search: String,
    /// The state of teh [`TextInput`](TextInput) of the search.
    #[focus(enable)]
    search_state: text_input::State,

    /// The state of the back [`Button`](Button).
    back_state: button::State,
    /// The state of the edit [`Button`](Button)
    edit_group_state: button::State,

    /// The state of the [`Scrollable`](iced::Scrollable).
    item_scroll_state: scrollable::State,

    /// The state of the [`Scrollable`](iced::Scrollable).
    tree_scroll_state: scrollable::State,

    /// The state of the [`Split`](Split).
    #[getset(get = "pub", get_mut = "pub")]
    split_state: split::State,

    /// A tree view of the groups.
    #[getset(get = "pub", get_mut = "pub")]
    group_tree: GroupTree,
}

/// The message that is send by the list view.
#[derive(Clone, Debug)]
pub enum ListMessage {
    /// Change the search to the new value.
    SearchInput(String),
    /// Go pack to the parent group.
    Back,
    /// Edit the currently selected group.
    EditGroup,
    /// Message that is send by the list items.
    ListItemMessage(ListItemMessage),
    /// The divider of the split is moved.
    SplitResize(u16),
    /// A message send by the group tree.
    GroupTreeMessage(GroupTreeMessage),
}
impl SomeIf for ListMessage {}

#[cfg_attr(test, mockable)]
impl ListView {
    /// Create a new [`ListView`](ListView).
    ///
    /// It expects:
    ///     - The UUID of the root group of the vault
    ///     - The number of sub-groups in the root group.
    ///     - The number of entries in the root group.
    pub fn new(root_uuid: Uuid, vault: &Vault) -> Self {
        let (group_count, entry_count) = (
            vault.get_groups_of(&root_uuid).len(),
            vault.get_entries_of(&root_uuid).len(),
        );

        let mut group_tree = GroupTree::new(root_uuid.clone(), vault);
        group_tree.toggle_expansion(vault);

        Self {
            selected_group_uuid: root_uuid,
            group_items: vec![ListGroupItem::default(); group_count],
            entry_items: vec![ListEntryItem::default(); entry_count],

            search: String::new(),
            search_state: text_input::State::focused(),

            back_state: button::State::new(),
            edit_group_state: button::State::new(),

            item_scroll_state: scrollable::State::new(),
            tree_scroll_state: scrollable::State::new(),
            split_state: split::State::new(Some(200), split::Axis::Vertical),

            group_tree,
        }
    }

    /// Resize the number of sub-groups and entries to the current configuration.
    pub fn resize(&mut self, vault: &Vault) {
        // TODO: remove
        let search = if self.search().is_empty() {
            None
        } else {
            Some(self.search().as_str())
        };
        let items = vault.get_item_list_for(&self.selected_group_uuid, search);
        let new_group_count = items.groups().len();
        let new_entry_count = items.entries().len();

        self.group_items = vec![ListGroupItem::default(); new_group_count];
        self.entry_items = vec![ListEntryItem::default(); new_entry_count];
    }

    /// Create the view of the [`ListView`](ListView).
    pub fn view<'a>(
        &'a mut self,
        vault: &'a Vault,
        theme: &dyn Theme,
        viewport: &Viewport,
    ) -> Element<'a, ListMessage> {
        let search_bar = TextInput::new(
            &mut self.search_state,
            "Search",
            &self.search,
            ListMessage::SearchInput,
        )
        .style(theme.text_input())
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let hide_group_tree = viewport.width < 600;

        let group_view = group_view(
            vault,
            &self.selected_group_uuid,
            &self.search,
            &mut self.back_state,
            &mut self.edit_group_state,
            &mut self.item_scroll_state,
            &mut self.group_items,
            &mut self.entry_items,
            theme,
            &crate::Viewport {
                width: if hide_group_tree {
                    viewport.width
                } else {
                    self.split_state.divider_position().map_or_else(
                        || viewport.width / 2,
                        |position| viewport.width - u32::from(position),
                    )
                },
                height: viewport.height,
            },
        );

        let content: Element<_> = if hide_group_tree {
            group_view
        } else {
            let tree_view = tree_view(
                vault,
                &mut self.tree_scroll_state,
                &mut self.group_tree,
                theme,
            );

            Split::new(
                &mut self.split_state,
                tree_view,
                group_view,
                ListMessage::SplitResize,
            )
            .style(theme.split())
            .padding(0.0)
            .into()
        };

        Container::new(
            Column::new()
                .push(search_bar)
                .push(vertical_space(2))
                .push(content),
        )
        .style(theme.container())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// Create the view of the group tree.
fn tree_view<'a>(
    vault: &'a Vault,
    scroll_state: &'a mut scrollable::State,
    group_tree: &'a mut GroupTree,
    theme: &dyn Theme,
) -> Element<'a, ListMessage> {
    Container::new(
        Scrollable::new(scroll_state)
            .push(
                group_tree
                    .view(0, vault, theme)
                    .map(ListMessage::GroupTreeMessage),
            )
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .style(theme.container_accent())
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Create the view of the group entries.
#[allow(clippy::too_many_arguments)]
fn group_view<'a>(
    vault: &'a Vault,
    selected_group_uuid: &Uuid,
    search: &str,
    back_state: &'a mut button::State,
    edit_group_state: &'a mut button::State,
    scroll_state: &'a mut scrollable::State,
    group_items: &'a mut [ListGroupItem],
    entry_items: &'a mut [ListEntryItem],
    theme: &dyn Theme,
    viewport: &Viewport,
) -> Element<'a, ListMessage> {
    let selected_group = vault.groups().get(selected_group_uuid).unwrap();

    //let icon_only = viewport.width < 1000; // TODO
    let icon_only = true;
    let no_buttons = viewport.width < 400;

    let current_item_list = vault.get_item_list_for(
        selected_group_uuid,
        if search.is_empty() {
            None
        } else {
            Some(search)
        },
    );

    let back = icon_button_with_width(
        ButtonData {
            state: back_state,
            icon: Icon::Backspace,
            text: "Back",
            kind: ButtonKind::Normal,
            on_press: ListMessage::Back.some_if_not(selected_group.is_root()),
        },
        "Go back to parent group",
        Length::Shrink,
        theme,
    );

    let edit_group = icon_button_with_width(
        ButtonData {
            state: edit_group_state,
            icon: Icon::Pencil,
            text: "Edit",
            kind: ButtonKind::Normal,
            on_press: ListMessage::EditGroup.some_if_not(selected_group.is_root()),
        },
        "Edit this group",
        Length::Shrink,
        theme,
    );

    let group_controls = Row::new()
        .spacing(2 * DEFAULT_ROW_SPACING)
        .align_items(iced::Align::Center)
        .push(back)
        .push(
            Text::new(if selected_group.is_root() {
                "Root"
            } else {
                selected_group.title()
            })
            .vertical_alignment(VerticalAlignment::Center)
            .width(Length::Fill),
        )
        .push(edit_group);

    let list: Element<_> = if current_item_list.is_empty() {
        Container::new(Text::new(if search.is_empty() {
            "This group is empty. Fill it by creating a new sub group or entry.".into()
        } else {
            format!("Could not find anything matching: {}", search)
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    } else {
        let mut list = Scrollable::new(scroll_state).spacing(DEFAULT_COLUMN_SPACING);

        list = group_items
            .iter_mut()
            .zip(current_item_list.groups().iter())
            .fold(list, |list, (item, group)| {
                list.push(item.view(group, theme).map(ListMessage::ListItemMessage))
            });

        list = entry_items
            .iter_mut()
            .zip(current_item_list.entries().iter())
            .fold(list, |list, (item, entry)| {
                list.push(
                    item.view(entry, icon_only, no_buttons, theme)
                        .map(ListMessage::ListItemMessage),
                )
            });

        list.into()
    };

    Container::new(
        Column::new()
            .push(group_controls)
            .push(default_vertical_space())
            .push(list),
    )
    //.style(theme.container_accent())
    .padding(5)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// The state of a sub-group list item.
#[derive(Clone, Debug, Default, Focus)]
struct ListGroupItem {
    /// The state of the [`Button`](Button) of the list item.
    state: button::State,
}

impl ListGroupItem {
    /// Create the view of the [`ListGroupItem`](ListGroupItem).
    fn view<'a>(&'a mut self, group: &'a Group, theme: &dyn Theme) -> Element<'a, ListItemMessage> {
        Button::new(
            &mut self.state,
            Row::new()
                .spacing(2 * DEFAULT_ROW_SPACING)
                .push(icon_text(Icon::Folder))
                .push(Text::new(group.title())),
        )
        .padding(20)
        .width(Length::Fill)
        .on_press(ListItemMessage::GroupSelected(group.uuid().clone()))
        .style(theme.list_item_group())
        .into()
    }
}

/// The state of an entry list item.
#[derive(Clone, Debug, Default, Focus)]
struct ListEntryItem {
    /// The state of the [`Button`](Button) of the list item.
    state: button::State,
    /// The state of the [`Button`](Button) to copy the username.
    copy_username_state: button::State,
    /// The state of the [`Button`](Button) to copy the password.
    copy_password_state: button::State,
    /// The state of the [`Button`](Button) to autofill the credentials.
    autofill_state: button::State,
}

impl ListEntryItem {
    /// Create the view of the [`ListEntryItem`](ListEntryItem).
    fn view<'a>(
        &'a mut self,
        entry: &'a EntryHead,
        icon_only: bool,
        no_buttons: bool,
        theme: &dyn Theme,
    ) -> Element<'a, ListItemMessage> {
        Button::new(
            &mut self.state,
            Row::new()
                .align_items(iced::Align::Center)
                .spacing(2 * DEFAULT_ROW_SPACING)
                .push(icon_text(Icon::Person))
                .push(Text::new(entry.title()).width(Length::Fill))
                .push(if no_buttons {
                    Row::new()
                } else {
                    Row::new()
                        .width(Length::Shrink)
                        .spacing(DEFAULT_ROW_SPACING)
                        .push(icon_button(
                            ButtonData {
                                state: &mut self.copy_username_state,
                                icon: Icon::FileEarmarkPerson,
                                text: "Username",
                                kind: ButtonKind::Normal,
                                on_press: Some(ListItemMessage::CopyUsername(entry.body().clone())),
                            },
                            "Copy username to clipboard",
                            icon_only,
                            theme,
                        ))
                        .push(icon_button(
                            ButtonData {
                                state: &mut self.copy_password_state,
                                icon: Icon::FileEarmarkLock,
                                text: "Password",
                                kind: ButtonKind::Normal,
                                on_press: Some(ListItemMessage::CopyPassword(entry.body().clone())),
                            },
                            "Copy password to clipboard",
                            icon_only,
                            theme,
                        ))
                        .push(icon_button(
                            ButtonData {
                                state: &mut self.autofill_state,
                                icon: Icon::Keyboard,
                                text: "AutoType",
                                kind: ButtonKind::Normal,
                                on_press: Some(ListItemMessage::Autofill(entry.uuid().clone())),
                            },
                            "Autofill credentials to the target window",
                            icon_only,
                            theme,
                        ))
                }),
        )
        .padding(20)
        .width(Length::Fill)
        .on_press(ListItemMessage::EntrySelected(entry.uuid().clone()))
        .style(theme.list_item_entry())
        .into()
    }
}

/// The message that is send by the list item.
#[derive(Clone, Debug)]
pub enum ListItemMessage {
    /// Select the group identified by it's UUID.
    GroupSelected(Uuid),
    /// Select the entry identified by it's UUID.
    EntrySelected(Uuid),
    /// Copy the username from the entry body identified by it's UUID.
    CopyUsername(Uuid),
    /// Copy the password from the entry body identified by it's UUID.
    CopyPassword(Uuid),
    /// Autofill credentials from the entry body identified by it's UUID  to the target.
    Autofill(Uuid),
}

/// A tree view of the group.
#[derive(Debug)]
pub struct GroupTree {
    /// The uuid of the group.
    group_uuid: Uuid,
    /// The cached title of the group.
    group_title: String,
    /// The children of this node.
    children: Vec<GroupTree>,

    /// The state of the tree node [`Button`](Button) of the [`GroupTree`](GroupTree).
    group_button: button::State,
    /// The state of the toggle [`Button`](Button) of the [`GroupTree`](GroupTree).
    toggle_button: button::State,
}

/// The message that is send by the group tree view.
#[derive(Clone, Debug)]
pub enum GroupTreeMessage {
    /// Toggle the expansion of a tree node. The node will be identified by it's path.
    ToggleExpansion(Vec<usize>),
    /// The user selected a group. It will be identified by it's uuid.
    GroupSelected(Uuid),
}

#[cfg_attr(test, mockable)]
impl GroupTree {
    /// Create a new [`GroupTree`](GroupTree).
    ///
    /// It expects:
    ///     - The UUID of the group to display.
    ///     - The vault to extract the group information from.
    pub fn new(group_uuid: Uuid, vault: &Vault) -> Self {
        let title = vault
            .groups()
            .get(&group_uuid)
            .map(|g| g.title().as_str())
            .filter(|t| !t.is_empty())
            .unwrap_or("Root");
        Self {
            group_uuid,
            group_title: title.to_owned(),
            children: Vec::new(),

            group_button: button::State::new(),
            toggle_button: button::State::new(),
        }
    }

    /// Toggle the expansion of the tree node.
    pub fn toggle_expansion(&mut self, vault: &Vault) {
        if self.children.is_empty() {
            self.children = vault
                .get_groups_of(&self.group_uuid)
                .iter()
                .map(|group| Self::new(group.uuid().clone(), vault))
                .collect();
            self.children
                .sort_by(|a, b| a.group_title.cmp(&b.group_title));
        } else {
            self.children.clear();
        }
    }

    /// Refresh the cached group information from the given vault.
    pub fn refresh(&mut self, vault: &Vault) {
        self.group_title = vault
            .groups()
            .get(&self.group_uuid)
            .map(|g| g.title().as_str())
            .filter(|t| !t.is_empty())
            .unwrap_or("Root")
            .to_owned();

        self.children
            .iter_mut()
            .for_each(|child| child.refresh(vault));

        // Check for newly added group: // TODO this is highly inefficient
        let sub_groups = vault.get_groups_of(&self.group_uuid);
        match self.children.len().cmp(&sub_groups.len()) {
            std::cmp::Ordering::Less => {
                let group_uuids: Vec<Uuid> = sub_groups.iter().map(|g| g.uuid().clone()).collect();

                let mut new_groups: Vec<Self> = group_uuids
                    .iter()
                    .filter(|uuid| {
                        !self
                            .children
                            .iter()
                            .map(|child| &child.group_uuid)
                            .any(|u| u == *uuid)
                    })
                    .fold(Vec::new(), |mut children, uuid| {
                        children.push(Self::new(uuid.clone(), vault));
                        children
                    });

                self.children.append(&mut new_groups);
                self.children
                    .sort_by(|a, b| a.group_title.cmp(&b.group_title));
            }
            std::cmp::Ordering::Greater => {
                let mut children_buffer = Vec::with_capacity(sub_groups.len());

                swap(&mut self.children, &mut children_buffer);

                for child in children_buffer {
                    if sub_groups
                        .iter()
                        .map(|g| g.uuid())
                        .any(|x| *x == child.group_uuid)
                    {
                        self.children.push(child);
                    }
                }
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    /// Update the group tree state.
    pub fn update(
        &mut self,
        message: GroupTreeMessage,
        vault: &Vault,
    ) -> Result<Command<GroupTreeMessage>, PWDuckGuiError> {
        match message {
            GroupTreeMessage::ToggleExpansion(mut stack) => {
                if stack.is_empty() {
                    self.toggle_expansion(vault);
                    Ok(Command::none())
                } else {
                    stack
                        .pop()
                        .and_then(|index| self.children.get_mut(index))
                        .ok_or(PWDuckGuiError::Option)?
                        .update(GroupTreeMessage::ToggleExpansion(stack), vault)
                }
            }
            GroupTreeMessage::GroupSelected(_) => {
                PWDuckGuiError::Unreachable("GroupTreeMessage".into()).into()
            }
        }
    }

    /// Create the view of the group tree node.
    pub fn view(
        &mut self,
        indentation: u16,
        vault: &Vault,
        theme: &dyn Theme,
    ) -> Element<GroupTreeMessage> {
        let content = Button::new(
            &mut self.group_button,
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .align_items(iced::Align::Center)
                .push(Space::new(Length::Units(indentation * 15), Length::Shrink))
                .push(
                    Button::new(
                        &mut self.toggle_button,
                        Text::new(if self.children.is_empty() {
                            Icon::PlusSquare
                        } else {
                            Icon::DashSquare
                        })
                        .font(ICON_FONT),
                    )
                    .on_press(GroupTreeMessage::ToggleExpansion(Vec::new()))
                    .padding(0)
                    .style(theme.tree_expand_button()),
                )
                .push(Text::new(&self.group_title)),
        )
        .width(Length::Fill)
        .on_press(GroupTreeMessage::GroupSelected(self.group_uuid.clone()))
        .style(theme.tree_node());

        let mut column = Column::new().width(Length::Fill).push(content);

        column = self
            .children
            .iter_mut()
            .enumerate()
            .fold(column, |col, (index, child)| {
                col.push(
                    child
                        .view(indentation + 1, vault, theme)
                        .map(move |msg| match msg {
                            GroupTreeMessage::ToggleExpansion(mut stack) => {
                                stack.push(index);
                                GroupTreeMessage::ToggleExpansion(stack)
                            }
                            GroupTreeMessage::GroupSelected(_) => msg,
                        }),
                )
            });

        column.into()
    }
}

#[cfg(test)]
mod tests {

    use pwduck_core::{uuid, MemKey, Uuid, Vault};
    use tempfile::{tempdir, TempDir};

    use std::{cell::RefCell, collections::HashMap};

    use mocktopus::mocking::*;

    use crate::error::PWDuckGuiError;

    use super::{GroupTree, GroupTreeMessage, ListView};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<String, usize>> = RefCell::new(HashMap::new());
    }

    const PASSWORD: &str = "this is a totally secret password";
    const DEFAULT_GROUP_COUNT: u8 = 15;
    const DEFAULT_ENTRY_COUNT: u8 = 15;

    const TOGGLE_EXPANSION: &str = "toggle_expansion";

    fn default_vault(mem_key: &MemKey) -> (TempDir, Vault) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");
        let mut vault = pwduck_core::Vault::generate(PASSWORD, mem_key, &path).unwrap();
        let master_key = vault
            .masterkey()
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
    fn new() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let list_view = ListView::new(root.clone(), &vault);

        assert_eq!(list_view.selected_group_uuid, root);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );
        assert!(list_view.search().is_empty());
        assert!(list_view.search_state.is_focused());
    }

    #[test]
    fn resize() {
        let mem_key = MemKey::with_length(1);
        let (_dir, mut vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut list_view = ListView::new(root.clone(), &vault);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );

        list_view.resize(&vault);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );

        list_view.search = "1".into();
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );
        list_view.resize(&vault);
        assert_eq!(list_view.group_items.len(), 6);
        assert_eq!(list_view.entry_items.len(), 6);

        list_view.search = "2".into();
        list_view.resize(&vault);
        assert_eq!(list_view.group_items.len(), 2);
        assert_eq!(list_view.entry_items.len(), 2);

        list_view.search = "Group".into();
        list_view.resize(&vault);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(list_view.entry_items.len(), 0);

        list_view.search = "Entry".into();
        list_view.resize(&vault);
        assert_eq!(list_view.group_items.len(), 0);
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );

        list_view.search = "".into();
        list_view.resize(&vault);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );

        let some_group_uuid: Uuid = [1; uuid::SIZE].into();

        list_view.selected_group_uuid = some_group_uuid.clone();
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&root).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&root).len()
        );
        list_view.resize(&vault);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&some_group_uuid).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&some_group_uuid).len()
        );

        // Add groups to some group
        for i in 0..3 {
            let i = DEFAULT_GROUP_COUNT + i;
            let group = pwduck_core::Group::new(
                [i; uuid::SIZE].into(),
                some_group_uuid.clone(),
                format!("Group: {}", i),
            );
            vault.insert_group(group);
        }

        let master_key = vault
            .masterkey()
            .as_unprotected(&mem_key, vault.salt(), vault.nonce())
            .unwrap();
        // Add entries to some group
        for i in 0..3 {
            let i = DEFAULT_ENTRY_COUNT + i;
            let head = pwduck_core::EntryHead::new(
                [i; uuid::SIZE].into(),
                some_group_uuid.clone(),
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
        assert_eq!(list_view.group_items.len(), 0);
        assert_eq!(list_view.entry_items.len(), 0);

        list_view.resize(&vault);
        assert_eq!(
            list_view.group_items.len(),
            vault.get_groups_of(&some_group_uuid).len()
        );
        assert_eq!(
            list_view.entry_items.len(),
            vault.get_entries_of(&some_group_uuid).len()
        );

        list_view.search = "Group".into();
        list_view.resize(&vault);
        assert_eq!(
            list_view.group_items.len(),
            DEFAULT_GROUP_COUNT as usize + 3
        );
        assert_eq!(list_view.entry_items.len(), 0);
    }

    #[test]
    fn new_group_tree() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let group_tree = GroupTree::new(root.clone(), &vault);
        assert_eq!(group_tree.group_uuid, root);
        assert_eq!(group_tree.group_title.as_str(), "Root");
        assert_eq!(group_tree.children.len(), 0);
    }

    #[test]
    fn toggle_expansion() {
        let mem_key = MemKey::with_length(1);
        let (_dir, vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut group_tree = GroupTree::new(root.clone(), &vault);
        assert_eq!(group_tree.children.len(), 0);

        group_tree.toggle_expansion(&vault);
        assert_eq!(group_tree.children.len(), DEFAULT_GROUP_COUNT as usize);

        group_tree.toggle_expansion(&vault);
        assert_eq!(group_tree.children.len(), 0);
    }

    #[test]
    fn refresh() {
        // TODO
    }

    #[test]
    fn update_group_tree() {
        let mem_key = MemKey::with_length(1);
        let (_dir, mut vault) = default_vault(&mem_key);
        let root = vault.get_root_uuid().unwrap();

        let mut group_tree = GroupTree::new(root.clone(), &vault);

        CALL_MAP.with(|call_map| unsafe {
            call_map.borrow_mut().insert(TOGGLE_EXPANSION.to_owned(), 0);

            GroupTree::toggle_expansion.mock_raw(|node, vault| {
                call_map
                    .borrow_mut()
                    .get_mut(TOGGLE_EXPANSION)
                    .map(|c| *c += 1);
                MockResult::Continue((node, vault))
            });

            assert!(group_tree.children.is_empty());
            assert_eq!(call_map.borrow()[TOGGLE_EXPANSION], 0);
            let _ = group_tree
                .update(GroupTreeMessage::ToggleExpansion(Vec::new()), &vault)
                .expect("Should not fail");
            assert_eq!(call_map.borrow()[TOGGLE_EXPANSION], 1);
            assert_eq!(group_tree.children.len(), DEFAULT_GROUP_COUNT as usize);

            let mut root_children = vault.get_groups_of(&root);
            root_children.sort_by(|a, b| a.title().cmp(b.title()));
            let roots_3rd_child_uuid = root_children.get(2).unwrap().uuid().clone();

            assert!(group_tree.children[2].children.is_empty());
            let _ = group_tree
                .update(GroupTreeMessage::ToggleExpansion(vec![2]), &vault)
                .expect("Should not fail");
            assert_eq!(call_map.borrow()[TOGGLE_EXPANSION], 2);
            assert!(group_tree.children[2].children.is_empty());

            for i in 0..5 {
                let i = DEFAULT_GROUP_COUNT + i;
                let group = pwduck_core::Group::new(
                    [i; uuid::SIZE].into(),
                    roots_3rd_child_uuid.clone(),
                    format!("Group: {}", i),
                );
                vault.insert_group(group)
            }

            let _ = group_tree
                .update(GroupTreeMessage::ToggleExpansion(vec![2]), &vault)
                .expect("Should not fail");
            assert_eq!(call_map.borrow()[TOGGLE_EXPANSION], 3);
            assert_eq!(group_tree.children[2].children.len(), 5);

            let mut roots_3rd_child_children = vault.get_groups_of(&roots_3rd_child_uuid);
            roots_3rd_child_children.sort_by(|a, b| a.title().cmp(b.title()));
            let roots_3rd_child_4th_child_uuid =
                roots_3rd_child_children.get(3).unwrap().uuid().clone();

            assert!(group_tree.children[2].children[3].children.is_empty());

            for i in 0..5 {
                let i = DEFAULT_GROUP_COUNT + 5 + i;
                let group = pwduck_core::Group::new(
                    [i; uuid::SIZE].into(),
                    roots_3rd_child_4th_child_uuid.clone(),
                    format!("Group: {}", i),
                );
                vault.insert_group(group);
            }

            let _ = group_tree
                .update(GroupTreeMessage::ToggleExpansion(vec![3, 2]), &vault)
                .expect("Should not fail");
            assert_eq!(call_map.borrow()[TOGGLE_EXPANSION], 4);
            assert_eq!(group_tree.children[2].children[3].children.len(), 5);

            let _ = group_tree
                .update(GroupTreeMessage::ToggleExpansion(vec![5, 4, 3, 2]), &vault)
                .expect_err("Should fail");
            assert_eq!(call_map.borrow()[TOGGLE_EXPANSION], 4);

            let res = group_tree
                .update(
                    GroupTreeMessage::GroupSelected([0; uuid::SIZE].into()),
                    &vault,
                )
                .expect_err("Should fail");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }
        });
    }
}
