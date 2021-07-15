//! TODO
use std::mem::swap;

use iced::{
    button, scrollable, text_input, Button, Column, Command, Container, Element, Length, Row,
    Scrollable, Space, Text, TextInput, VerticalAlignment,
};
use iced_aw::{split, Split};
use pwduck_core::{EntryHead, Group, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::{Icon, ICON_FONT},
    utils::{
        default_vertical_space, icon_button, icon_button_with_width, icon_text, vertical_space,
        SomeIf,
    },
    Viewport, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};
use getset::{Getters, MutGetters, Setters};

/// The state of the list view inside the vault container.
///
/// See: [`VaultContainer`](crate::vault::container::VaultContainer)
#[derive(Debug, Getters, MutGetters, Setters)]
pub struct ListView {
    /// The UUID of the selected group.
    #[getset(get = "pub", set = "pub")]
    selected_group_uuid: String,
    /// The sub-groups of the selected group.
    #[getset(get)]
    group_items: Vec<ListGroupItem>,
    /// The entries of the selected group.
    #[getset(get)]
    entry_items: Vec<ListEntryItem>,

    /// The search string to search for groups / entries.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    search: String,
    /// The state of teh [`TextInput`](TextInput) of the search.
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

impl ListView {
    /// Create a new [`ListView`](ListView).
    ///
    /// It expects:
    ///     - The UUID of the root group of the vault
    ///     - The number of sub-groups in the root group.
    ///     - The number of entries in the root group.
    pub fn new(root_uuid: String, vault: &Vault) -> Self {
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
            search_state: text_input::State::new(),

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
        viewport: &Viewport,
    ) -> Element<'a, ListMessage> {
        let search_bar = TextInput::new(
            &mut self.search_state,
            "Search",
            &self.search,
            ListMessage::SearchInput,
        )
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
            let tree_view = tree_view(vault, &mut self.tree_scroll_state, &mut self.group_tree);

            Split::new(
                &mut self.split_state,
                tree_view,
                group_view,
                ListMessage::SplitResize,
            )
            .padding(5.0)
            .into()
        };

        Container::new(
            Column::new()
                .push(search_bar)
                .push(vertical_space(2))
                .push(content),
        )
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
) -> Element<'a, ListMessage> {
    Scrollable::new(scroll_state)
        .push(group_tree.view(0, vault).map(ListMessage::GroupTreeMessage))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Create the view of the group entries.
#[allow(clippy::too_many_arguments)]
fn group_view<'a>(
    vault: &'a Vault,
    selected_group_uuid: &str,
    search: &str,
    back_state: &'a mut button::State,
    edit_group_state: &'a mut button::State,
    scroll_state: &'a mut scrollable::State,
    group_items: &'a mut [ListGroupItem],
    entry_items: &'a mut [ListEntryItem],
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
        back_state,
        Icon::Backspace,
        "Back",
        "Go to parent group",
        ListMessage::Back.some_if_not(selected_group.is_root()),
        Length::Shrink,
    );

    let edit_group = icon_button_with_width(
        edit_group_state,
        Icon::Pencil,
        "Edit",
        "Edit this group",
        ListMessage::EditGroup.some_if_not(selected_group.is_root()),
        Length::Shrink,
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
                list.push(item.view(group).map(ListMessage::ListItemMessage))
            });

        list = entry_items
            .iter_mut()
            .zip(current_item_list.entries().iter())
            .fold(list, |list, (item, entry)| {
                list.push(
                    item.view(entry, icon_only, no_buttons)
                        .map(ListMessage::ListItemMessage),
                )
            });

        list.into()
    };

    Column::new()
        .push(group_controls)
        .push(default_vertical_space())
        .push(list)
        .into()
}

/// The state of a sub-group list item.
#[derive(Clone, Debug, Default)]
struct ListGroupItem {
    /// The state of the [`Button`](Button) of the list item.
    state: button::State,
}

impl ListGroupItem {
    /// Create the view of the [`ListGroupItem`](ListGroupItem).
    fn view<'a>(&'a mut self, group: &'a Group) -> Element<'a, ListItemMessage> {
        Button::new(
            &mut self.state,
            Row::new()
                .spacing(2 * DEFAULT_ROW_SPACING)
                .push(icon_text(Icon::Folder))
                .push(Text::new(group.title())),
        )
        .padding(20)
        .width(Length::Fill)
        .on_press(ListItemMessage::GroupSelected(group.uuid().as_string()))
        .style(ListGroupStyle)
        .into()
    }
}

/// The state of an entry list item.
#[derive(Clone, Debug, Default)]
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
                            &mut self.copy_username_state,
                            Icon::FileEarmarkPerson,
                            "Username",
                            "Copy username",
                            icon_only,
                            Some(ListItemMessage::CopyUsername(entry.body().clone())),
                        ))
                        .push(icon_button(
                            &mut self.copy_password_state,
                            Icon::FileEarmarkLock,
                            "Password",
                            "Copy password",
                            icon_only,
                            Some(ListItemMessage::CopyPassword(entry.body().clone())),
                        ))
                        .push(icon_button(
                            &mut self.autofill_state,
                            Icon::Keyboard,
                            "AutoType",
                            "Autofill credentials to the target window",
                            icon_only,
                            None,
                        ))
                }),
        )
        .padding(20)
        .width(Length::Fill)
        .on_press(ListItemMessage::EntrySelected(entry.uuid().as_string()))
        .style(ListEntryStyle)
        .into()
    }
}

/// The message that is send by the list item.
#[derive(Clone, Debug)]
pub enum ListItemMessage {
    /// Select the group identified by it's UUID.
    GroupSelected(String),
    /// Select the entry identified by it's UUID.
    EntrySelected(String),
    /// Copy the username from the entry body identified by it's UUID.
    CopyUsername(String),
    /// Copy the password from the entry body identified by it's UUID.
    CopyPassword(String),
    /// Autofill credentials from the entry body identified by it's UUID  to the target.
    Autofill(String),
}

/// The style of the [`ListGroupItem`](ListGroupItem)s.
#[derive(Debug, Default)]
struct ListGroupStyle;

impl button::StyleSheet for ListGroupStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::default(),
            background: None,
            border_radius: 5.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
            text_color: iced::Color::BLACK,
        }
    }
}

/// The style of the [`ListEntryItem`](ListEntryItem)s.
#[derive(Debug, Default)]
struct ListEntryStyle;

impl button::StyleSheet for ListEntryStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::default(),
            background: None,
            border_radius: 5.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
            text_color: iced::Color::BLACK,
        }
    }
}

/// A tree view of the group.
#[derive(Debug)]
pub struct GroupTree {
    /// The uuid of the group.
    group_uuid: String,
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
    GroupSelected(String),
}

impl GroupTree {
    /// Create a new [`GroupTree`](GroupTree).
    ///
    /// It expects:
    ///     - The UUID of the group to display.
    ///     - The vault to extract the group information from.
    pub fn new(group_uuid: String, vault: &Vault) -> Self {
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
                .map(|group| Self::new(group.uuid().as_string(), vault))
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
                let group_uuids: Vec<String> =
                    sub_groups.iter().map(|g| g.uuid().as_string()).collect();

                let mut new_groups: Vec<Self> = group_uuids
                    .iter()
                    .filter(|uuid| {
                        !self
                            .children
                            .iter()
                            .map(|child| child.group_uuid.as_str())
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
                let group_uuids: Vec<String> =
                    sub_groups.iter().map(|g| g.uuid().as_string()).collect();

                swap(&mut self.children, &mut children_buffer);

                for child in children_buffer {
                    if group_uuids.contains(&child.group_uuid) {
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
                    let index = stack.pop().ok_or(PWDuckGuiError::Option)?;

                    self.children[index].update(GroupTreeMessage::ToggleExpansion(stack), vault)
                }
            }
            GroupTreeMessage::GroupSelected(_) => {
                PWDuckGuiError::Unreachable("GroupTreeMessage".into()).into()
            }
        }
    }

    /// Create the view of the group tree node.
    pub fn view(&mut self, indentation: u16, vault: &Vault) -> Element<GroupTreeMessage> {
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
                    .style(GroupTreeToggleStyle),
                )
                .push(Text::new(&self.group_title)),
        )
        .width(Length::Fill)
        .on_press(GroupTreeMessage::GroupSelected(self.group_uuid.clone()))
        .style(GroupTreeStyle);

        let mut column = Column::new().width(Length::Fill).push(content);

        column = self
            .children
            .iter_mut()
            .enumerate()
            .fold(column, |col, (index, child)| {
                col.push(
                    child
                        .view(indentation + 1, vault)
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

/// The style of the group tree.
#[derive(Clone, Copy, Debug)]
struct GroupTreeStyle;

impl button::StyleSheet for GroupTreeStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::new(0.0, 0.0),
            background: iced::Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::BLACK,
        }
    }
}

/// The style of the group tree toggle button.
#[derive(Clone, Copy, Debug)]
struct GroupTreeToggleStyle;

impl button::StyleSheet for GroupTreeToggleStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::new(0.0, 0.0),
            background: iced::Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::BLACK,
        }
    }
}
