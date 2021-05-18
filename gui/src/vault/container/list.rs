use iced::{
    button, container, scrollable, text_input, Button, Column, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Space, Text, TextInput, VerticalAlignment,
};
use pwduck_core::{EntryHead, Group, Vault};

use crate::{
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_HEADER_SIZE, DEFAULT_ROW_SPACING,
    DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING,
};
use getset::{Getters, Setters};

#[derive(Debug, Getters, Setters)]
pub struct ListView {
    #[getset(get = "pub", set = "pub")]
    selected_group_uuid: String,
    #[getset(get)]
    group_items: Vec<ListGroupItem>,
    #[getset(get)]
    entry_items: Vec<ListEntryItem>,

    save_state: button::State,
    new_group_state: button::State,
    new_entry_state: button::State,
    copy_username_state: button::State,
    copy_password_state: button::State,
    lock_vault_state: button::State,

    #[getset(get = "pub", set = "pub")]
    search: String,
    search_state: text_input::State,

    back_state: button::State,

    scroll_state: scrollable::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum ListMessage {
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
    SearchInput(String),
    /// TODO
    Back,
    /// TODO
    ListItemMessage(ListItemMessage),
}

impl ListView {
    pub fn new(root_uuid: String, group_count: usize, entry_count: usize) -> Self {
        Self {
            selected_group_uuid: root_uuid,
            group_items: vec![ListGroupItem::default(); group_count],
            entry_items: vec![ListEntryItem::default(); entry_count],

            save_state: button::State::new(),
            new_group_state: button::State::new(),
            new_entry_state: button::State::new(),
            copy_username_state: button::State::new(),
            copy_password_state: button::State::new(),
            lock_vault_state: button::State::new(),

            search: String::new(),
            search_state: text_input::State::new(),

            back_state: button::State::new(),

            scroll_state: scrollable::State::new(),
        }
    }

    pub fn resize(&mut self, vault: &Vault) {
        let (new_group_count, new_entry_count) = (
            vault.get_groups_of(&self.selected_group_uuid).len(),
            vault.get_entries_of(&self.selected_group_uuid).len(),
        );

        self.group_items = vec![ListGroupItem::default(); new_group_count];
        self.entry_items = vec![ListEntryItem::default(); new_entry_count];
    }

    pub fn view<'a>(&'a mut self, vault: &'a Vault) -> Element<'a, ListMessage> {
        let vault_contains_unsaved_changes = vault.contains_unsaved_changes();
        let current_item_list = vault.get_item_list_for(
            &self.selected_group_uuid,
            if self.search.is_empty() {
                None
            } else {
                Some(&self.search)
            },
        );

        let mut save = Button::new(
            &mut self.save_state,
            Text::new("Save Vault")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);
        if vault_contains_unsaved_changes {
            save = save.on_press(ListMessage::Save);
        }

        let new_group = Button::new(
            &mut self.new_group_state,
            Text::new("New Group")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .on_press(ListMessage::NewGroup)
        .width(Length::Fill);

        let new_entry = Button::new(
            &mut self.new_entry_state,
            Text::new("New Entry")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .on_press(ListMessage::NewEntry)
        .width(Length::Fill);

        let copy_username = Button::new(
            &mut self.copy_username_state,
            Text::new("Copy username")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);

        let copy_password = Button::new(
            &mut self.copy_password_state,
            Text::new("Copy password")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);

        let mut lock_vault = Button::new(
            &mut self.lock_vault_state,
            Text::new("Lock Vault")
                .horizontal_alignment(HorizontalAlignment::Center)
                .width(Length::Fill),
        )
        .width(Length::Fill);

        if !vault_contains_unsaved_changes {
            lock_vault = lock_vault.on_press(ListMessage::LockVault)
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

        let selected_group = vault.groups().get(&self.selected_group_uuid).unwrap();

        let search_bar = TextInput::new(
            &mut self.search_state,
            "Search",
            &self.search,
            ListMessage::SearchInput,
        )
        .padding(DEFAULT_TEXT_INPUT_PADDING);

        let mut back = Button::new(&mut self.back_state, Text::new("< Back"));
        if !selected_group.is_root() {
            back = back.on_press(ListMessage::Back);
        }

        let list: Element<_> = if current_item_list.is_empty() {
            Container::new(Text::new(if self.search.is_empty() {
                "This group is empty. Fill it by creating a new sub group or entry.".into()
            } else {
                format!("Could not find anything matching: {}", self.search)
            }))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            let mut list = Scrollable::new(&mut self.scroll_state).spacing(DEFAULT_COLUMN_SPACING);

            list = self
                .group_items
                .iter_mut()
                .zip(current_item_list.groups().iter())
                .fold(list, |list, (item, group)| {
                    list.push(item.view(group).map(ListMessage::ListItemMessage))
                });

            list = self
                .entry_items
                .iter_mut()
                .zip(current_item_list.entries().iter())
                .fold(list, |list, (item, entry)| {
                    list.push(item.view(entry).map(ListMessage::ListItemMessage))
                });

            list.into()
        };

        Container::new(
            Column::new()
                .padding(DEFAULT_COLUMN_PADDING)
                .spacing(DEFAULT_ROW_SPACING)
                .push(Text::new(&format!("Vault: {}", &vault.get_name())).size(DEFAULT_HEADER_SIZE))
                .push(toolbar)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(search_bar)
                .push(Space::with_height(Length::Units(DEFAULT_SPACE_HEIGHT)))
                .push(
                    Row::new()
                        .spacing(DEFAULT_ROW_SPACING)
                        .align_items(iced::Align::Center)
                        .push(back)
                        .push(
                            Text::new(if selected_group.is_root() {
                                "Root"
                            } else {
                                selected_group.title()
                            })
                            .vertical_alignment(VerticalAlignment::Center),
                        ),
                )
                .push(list),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Clone, Debug, Default)]
struct ListGroupItem {
    state: button::State,
}

impl ListGroupItem {
    fn view<'a>(&'a mut self, group: &'a Group) -> Element<'a, ListItemMessage> {
        Button::new(
            &mut self.state,
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(Text::new(group.title())),
        )
        .padding(20)
        .width(Length::Fill)
        .on_press(ListItemMessage::GroupSelected(group.uuid().as_string()))
        .style(ListGroupStyle)
        .into()
    }
}

#[derive(Clone, Debug, Default)]
struct ListEntryItem {
    state: button::State,
}

impl ListEntryItem {
    fn view<'a>(&'a mut self, entry: &'a EntryHead) -> Element<'a, ListItemMessage> {
        Button::new(
            &mut self.state,
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(Text::new(entry.title())),
        )
        .padding(20)
        .width(Length::Fill)
        .on_press(ListItemMessage::EntrySelected(entry.uuid().as_string()))
        .style(ListEntryStyle)
        .into()
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum ListItemMessage {
    GroupSelected(String),
    EntrySelected(String),
}

#[derive(Debug, Default)]
struct ListGroupStyle;

impl button::StyleSheet for ListGroupStyle {
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

#[derive(Debug, Default)]
struct ListEntryStyle;

impl button::StyleSheet for ListEntryStyle {
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
