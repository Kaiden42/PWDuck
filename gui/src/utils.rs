//! TODO

use iced::{button, Button, Length, Text};

/// TODO
pub fn icon_button<Message: Clone>(
    state: &mut button::State,
    _icon: impl Into<String>,
    text: impl Into<String>,
) -> Button<Message> {
    Button::new(
        state,
        horizontal_text(text), // replace with row for icons
    )
    .width(Length::Fill)
}

/// TODO
pub fn horizontal_text(label: impl Into<String>) -> Text {
    Text::new(label)
        .horizontal_alignment(iced::HorizontalAlignment::Center)
        .width(Length::Fill)
}
