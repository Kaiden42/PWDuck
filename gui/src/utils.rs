//! TODO

use iced::{
    button, text_input, Button, Column, Container, Element, Length, Space, Text, TextInput,
};

use crate::{
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH, DEFAULT_SPACE_HEIGHT,
    DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
pub fn icon_button<Message: Clone>(
    state: &mut button::State,
    _icon: impl Into<String>,
    text: impl Into<String>,
) -> Button<Message> {
    Button::new(
        state,
        horizontal_centered_text(text), // replace with row for icons
    )
    .width(Length::Fill)
}

/// TODO
pub fn horizontal_centered_text(label: impl Into<String>) -> Text {
    Text::new(label)
        .horizontal_alignment(iced::HorizontalAlignment::Center)
        .width(Length::Fill)
}

/// TODO
pub fn default_text_input<'a, F, Message: Clone>(
    state: &'a mut text_input::State,
    placeholder: &str,
    value: &str,
    on_change: F,
) -> TextInput<'a, Message>
where
    F: 'static + Fn(String) -> Message,
{
    TextInput::new(state, placeholder, value, on_change).padding(DEFAULT_TEXT_INPUT_PADDING)
}

/// TODO
pub fn centered_container_with_column<'a, Message: 'a>(
    children: Vec<Element<'a, Message>>,
) -> Container<'a, Message> {
    Container::new(
        Column::with_children(children)
            .max_width(DEFAULT_MAX_WIDTH)
            .padding(DEFAULT_COLUMN_PADDING)
            .spacing(DEFAULT_COLUMN_SPACING),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
}

/// TODO
pub fn default_vertical_space() -> Space {
    vertical_space(1)
}

/// TODO
pub fn vertical_space(factor: u16) -> Space {
    Space::with_height(Length::Units(factor * DEFAULT_SPACE_HEIGHT))
}
