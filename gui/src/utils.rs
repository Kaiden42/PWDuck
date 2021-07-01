//! Utility functions.

use iced::{
    button, container, text_input, tooltip, Button, Column, Container, Element, Length, Row, Space,
    Text, TextInput, Tooltip,
};

use crate::{
    icons::{Icon, ICON_FONT},
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING,
    DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING,
};

/// Create a [`Button`] with an [`Icon`](Icon) and a [`Text`](iced::Text).
///
/// It expects:
///     - The [`State`](button::State) of the [`Button`](Button)
///     - The [`Icon`](Icon) of the [`Button`](Button)
///     - The text of the [`Button`](Button)
///     - The tooltip of the [`Button`](Button)
///     - If only the [`Icon`](Icon) of the [`Button`](Button) should be visible
///     - The message that the [`Button`](Button) sends if the user presses on it
pub fn icon_button<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    icon: Icon,
    text: impl Into<String>,
    tooltip: impl Into<String>,
    icon_only: bool,
    on_press: Option<Message>,
) -> Element<Message> {
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

    let mut button = Button::new(state, element).width(if icon_only {
        Length::Shrink
    } else {
        Length::Fill
    });

    if let Some(message) = on_press {
        button = button.on_press(message);
    }

    Tooltip::new(button, tooltip.into(), tooltip::Position::FollowCursor)
        .style(TooltipStyle)
        .into()
}

/// Create a [`Button`](Button) with an [`Icon`](Icon), a [`Text`](iced::Text) and a specified width.
///
/// It expects:
///     - The [`State`](button::State) of the [`Button`](Button)
///     - The [`Icon`](Icon) of the [`Button`](Button)
///     - The text of the [`Button`](Button)
///     - The tooltip of the [`Button`](Button)
///     - The message that the [`Button`](Button) sends if the user presses on it
///     - The width of the [`Button`](Button)
pub fn icon_button_with_width<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    icon: Icon,
    text: impl Into<String>,
    tooltip: impl Into<String>,
    on_press: Option<Message>,
    width: Length,
) -> Element<Message> {
    let element: Element<_> = Container::new(
        Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .push(icon_text(icon))
            //.push(horizontal_centered_text(text))
            .push(Text::new(text))
            .width(Length::Shrink),
    )
    .width(Length::FillPortion(1))
    .align_x(iced::Align::Center)
    .into();

    let mut button = Button::new(state, element).width(width);

    if let Some(message) = on_press {
        button = button.on_press(message);
    }

    Tooltip::new(button, tooltip.into(), tooltip::Position::FollowCursor)
        .style(TooltipStyle)
        .into()
}

/// Create a label containing an Icon with the default `ICON_FONT`.
pub fn icon_text(icon: Icon) -> Text {
    Text::new(icon).width(Length::Shrink).font(ICON_FONT)
}

/// Create a toggle button to toggle the password visibility.
///
/// It expects:
///     - The [`State`](button::State) of the [`Button`](Button)
///     - The state of the password visibility
///     - The message that the [`Button`](Button) sends if the user presses on it
pub fn password_toggle<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    show_password: bool,
    on_press: Message,
) -> Element<Message> {
    if show_password {
        icon_button(
            state,
            Icon::EyeSlash,
            "Hide",
            "Hide password",
            true,
            Some(on_press),
        )
    } else {
        icon_button(
            state,
            Icon::Eye,
            "Show",
            "Show password",
            true,
            Some(on_press),
        )
    }
}

/// Create text that is horizontally centered.
pub fn horizontal_centered_text(label: impl Into<String>) -> Text {
    Text::new(label)
        .horizontal_alignment(iced::HorizontalAlignment::Center)
        .width(Length::Fill)
}

/// Create a default [`TextInput`](TextInput).
///
/// It expects:
///     - The [`State`](text_input::State) of the [`TextInput`](TextInput)
///     - The placeholder of the [`TextInput`](TextInput)
///     - The value of the [`TextInput`](TextInput)
///     - The message that the [`TextInput`](TextInput) sends if the value is changed
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

/// Create a default container.
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

/// Create a default vertical [`Space`](Space).
pub fn default_vertical_space() -> Space {
    vertical_space(1)
}

/// Create a vertical [`Space`] that is `factor` times larger than the default [`Space`](Space).
pub fn vertical_space(factor: u16) -> Space {
    Space::with_height(Length::Units(factor * DEFAULT_SPACE_HEIGHT))
}

/// Calculate the strength of the given password.
pub async fn estimate_password_strength(
    password: pwduck_core::SecString,
) -> Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError> {
    pwduck_core::password_entropy(&password)
}

/// Shortcut trait to create `Some(value)` or `None` based on a condition.
pub trait SomeIf {
    /// Returns `Some(self)` if the `condition` is true, `None` if not.
    fn some_if(self, condition: bool) -> Option<Self>
    where
        Self: Sized,
    {
        if condition {
            Some(self)
        } else {
            None
        }
    }

    /// Returns `Some(self)` if the `condition` is not true, `None` if it is.
    fn some_if_not(self, condition: bool) -> Option<Self>
    where
        Self: Sized,
    {
        if condition {
            None
        } else {
            Some(self)
        }
    }
}

/// The default style of a [`Tooltip`](iced::Tooltip).
#[derive(Clone, Copy, Debug, Default)]
pub struct TooltipStyle;

impl container::StyleSheet for TooltipStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(iced::Color::BLACK),
            background: iced::Color::WHITE.into(),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
        }
    }
}
