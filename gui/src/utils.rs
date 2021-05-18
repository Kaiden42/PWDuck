//! TODO

use iced::{Button, Length, Text, button};

pub fn icon_button<'a, Message: Clone>(
    state: &'a mut button::State,
    _icon: impl Into<String>,
    text: impl Into<String>,
) -> Button<'a, Message> {
    Button::new(
        state,
        horizontal_text(text), // replace with row for icons
    )
    .width(Length::Fill)
}

pub fn horizontal_text(
    label: impl Into<String>,
) -> Text {
    Text::new(label)
        .horizontal_alignment(iced::HorizontalAlignment::Center)
        .width(Length::Fill)
}