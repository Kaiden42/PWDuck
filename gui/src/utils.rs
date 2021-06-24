//! TODO

use iced::{
    button, text_input, Button, Column, Container, Element, Length, Row, Space, Text, TextInput,
};

use crate::{
    icons::{Icon, ICON_FONT},
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING,
    DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING,
};

/// TODO
pub fn icon_button<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    icon: Icon,
    text: impl Into<String>,
    tooltip: impl Into<String>,
    icon_only: bool,
) -> Button<Message> {
    let element: Element<_> = if icon_only {
        icon_text(icon).into()
    } else {
        Container::new(
            Row::new()
                .spacing(DEFAULT_ROW_SPACING)
                .push(icon_text(icon))
                //.push(horizontal_centered_text(text))
                .push(Text::new(text))
                .width(Length::Shrink),
        )
        .width(Length::FillPortion(1))
        .align_x(iced::Align::Center)
        .into()
    };

    Button::new(state, element).width(if icon_only {
        Length::Shrink
    } else {
        Length::Fill
    })
}

pub fn icon_text(icon: Icon) -> Text {
    Text::new(icon).width(Length::Shrink).font(ICON_FONT)
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

/// TODO
pub async fn estimate_password_strength(
    password: pwduck_core::SecString,
) -> Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError> {
    pwduck_core::password_entropy(&password)
}
