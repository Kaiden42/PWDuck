//! TODO
use iced::{
    button, scrollable, text_input, Button, Column, Container, Element, Length, Row, Scrollable,
    Text, TextInput, VerticalAlignment,
};
use pwduck_core::{EntryHead, Group, Vault};

use crate::{
    utils::{default_vertical_space, vertical_space},
    DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING, DEFAULT_TEXT_INPUT_PADDING,
};
use getset::{Getters, Setters};

/// TODO
#[derive(Debug, Getters, Setters)]
pub struct ListView {
    /// TODO
    #[getset(get = "pub", set = "pub")]
    selected_group_uuid: String,
    /// TODO
    #[getset(get)]
    group_items: Vec<ListGroupItem>,
    /// TODO
    #[getset(get)]
    entry_items: Vec<ListEntryItem>,

    /// TODO
    #[getset(get = "pub", set = "pub")]
    search: String,
    /// TODO
    search_state: text_input::State,

    /// TODO
    back_state: button::State,

    /// TODO
    scroll_state: scrollable::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum ListMessage {
    /// TODO
    SearchInput(String),
    /// TODO
    Back,
    /// TODO
    ListItemMessage(ListItemMessage),
}

impl ListView {
    /// TODO
    pub fn new(root_uuid: String, group_count: usize, entry_count: usize) -> Self {
        Self {
            selected_group_uuid: root_uuid,
            group_items: vec![ListGroupItem::default(); group_count],
            entry_items: vec![ListEntryItem::default(); entry_count],

            search: String::new(),
            search_state: text_input::State::new(),

            back_state: button::State::new(),

            scroll_state: scrollable::State::new(),
        }
    }

    /// TODO
    pub fn resize(&mut self, vault: &Vault) {
        let (new_group_count, new_entry_count) = (
            vault.get_groups_of(&self.selected_group_uuid).len(),
            vault.get_entries_of(&self.selected_group_uuid).len(),
        );

        self.group_items = vec![ListGroupItem::default(); new_group_count];
        self.entry_items = vec![ListEntryItem::default(); new_entry_count];
    }

    /// TODO
    pub fn view<'a>(&'a mut self, vault: &'a Vault) -> Element<'a, ListMessage> {
        let current_item_list = vault.get_item_list_for(
            &self.selected_group_uuid,
            if self.search.is_empty() {
                None
            } else {
                Some(&self.search)
            },
        );

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
                .push(search_bar)
                .push(vertical_space(2))
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
                .push(default_vertical_space())
                .push(list),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// TODO
#[derive(Clone, Debug, Default)]
struct ListGroupItem {
    /// TODO
    state: button::State,
}

impl ListGroupItem {
    /// TODO
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

/// TODO
#[derive(Clone, Debug, Default)]
struct ListEntryItem {
    /// TODO
    state: button::State,
}

impl ListEntryItem {
    /// TODO
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
    /// TODO
    GroupSelected(String),
    /// TODO
    EntrySelected(String),
}

/// TODO
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

/// TODO
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
