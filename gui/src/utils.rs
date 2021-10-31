//! Utility functions.

use iced::{
    button, text_input, tooltip, Button, Column, Container, Element, Length, Row, Space, Text,
    TextInput, Tooltip,
};

use crate::{
    icons::{Icon, ICON_FONT},
    theme::Theme,
    DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH, DEFAULT_ROW_SPACING,
    DEFAULT_SPACE_HEIGHT, DEFAULT_TEXT_INPUT_PADDING,
};

/// Helper struct to reduce function argument count.
pub struct ButtonData<'a, Message: 'a + Clone> {
    /// The [`State`](button::State) of the [`Button`](Button).
    pub state: &'a mut button::State,
    /// The icon of the [`Button`](Button).
    pub icon: Icon,
    /// The text of the [`Button`](Button).
    pub text: &'a str,
    /// The kind of a [`Button`](Button).
    pub kind: ButtonKind,
    /// The optional message that is send by the [`Button`](Button).
    pub on_press: Option<Message>,
}

impl<'a, Message: 'a + Clone> ButtonData<'a, Message> {
    /// Create the element containing only the icon.
    #[cfg_attr(coverage, no_coverage)]
    pub fn icon_element(&self) -> Element<'a, Message> {
        icon_text(self.icon).into()
    }

    /// Create the element containing the icon and the text.
    #[cfg_attr(coverage, no_coverage)]
    pub fn icon_text_element(&self) -> Element<'a, Message> {
        Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .push(icon_text(self.icon))
            .push(Text::new(self.text))
            .width(Length::Shrink)
            .into()
    }
}

/// The kind of a [`Button`](Button).
pub enum ButtonKind {
    /// The button is a normal [`Button`](Button).
    Normal,
    /// The button is a primary [`Button`](Button).
    Primary,
    /// The button is a warning [`Button`](Button).
    Warning,
}

impl ButtonKind {
    /// Returns the style sheet for the specific [`ButtonKind`](ButtonKind).
    #[cfg_attr(coverage, no_coverage)]
    pub fn style_sheet(&self, theme: &dyn Theme) -> Box<dyn button::StyleSheet> {
        match self {
            ButtonKind::Normal => theme.button(),
            ButtonKind::Primary => theme.button_primary(),
            ButtonKind::Warning => theme.button_warning(),
        }
    }
}

/// Create a [`Button`] with an [`Icon`](Icon) and a [`Text`](iced::Text).
///
/// It expects:
///  - The data of the [`Button`](Button).
///  - The tooltip of the [`Button`](Button).
///  - If only the [`Icon`](Icon) of the [`Button`](Button) should be visible.
///  - The theme of the application.
#[cfg_attr(coverage, no_coverage)]
pub fn icon_button<'a, Message: 'a + Clone>(
    button_data: ButtonData<'a, Message>,
    tooltip: impl Into<String>,
    icon_only: bool,
    theme: &dyn Theme,
) -> Element<'a, Message> {
    let element: Element<_> = if icon_only {
        button_data.icon_element()
    } else {
        Container::new(button_data.icon_text_element())
            .width(Length::FillPortion(1))
            .align_x(iced::Align::Center)
            .into()
    };

    let mut button = Button::new(button_data.state, element)
        .width(if icon_only {
            Length::Shrink
        } else {
            Length::Fill
        })
        .style(button_data.kind.style_sheet(theme));

    if let Some(message) = button_data.on_press {
        button = button.on_press(message);
    }

    Tooltip::new(button, tooltip.into(), tooltip::Position::FollowCursor)
        .style(theme.tooltip())
        .into()
}

/// Create a [`Button`](Button) with an [`Icon`](Icon), a [`Text`](iced::Text) and a specified width.
///
/// It expects:
///  - The data of the [`Button`](Button).
///  - The tooltip of the [`Button`](Button).
///  - The width of the [`Button`](Button).
///  - The theme of the application.
#[cfg_attr(coverage, no_coverage)]
pub fn icon_button_with_width<'a, Message: 'a + Clone>(
    button_data: ButtonData<'a, Message>,
    tooltip: impl Into<String>,
    width: Length,
    theme: &dyn Theme,
) -> Element<'a, Message> {
    let element: Element<_> = Container::new(button_data.icon_text_element())
        .width(Length::FillPortion(1))
        .align_x(iced::Align::Center)
        .into();

    let mut button = Button::new(button_data.state, element)
        .width(width)
        .style(button_data.kind.style_sheet(theme));

    if let Some(message) = button_data.on_press {
        button = button.on_press(message);
    }

    Tooltip::new(button, tooltip.into(), tooltip::Position::FollowCursor)
        .style(theme.tooltip())
        .into()
}

/// Create a label containing an Icon with the default `ICON_FONT`.
#[cfg_attr(coverage, no_coverage)]
pub fn icon_text(icon: Icon) -> Text {
    Text::new(icon).width(Length::Shrink).font(ICON_FONT)
}

/// Create a toggle button to toggle the password visibility.
///
/// It expects:
///  - The [`State`](button::State) of the [`Button`](Button)
///  - The state of the password visibility
///  - The message that the [`Button`](Button) sends if the user presses on it
///  - The theme of the application.
#[cfg_attr(coverage, no_coverage)]
pub fn password_toggle<'a, Message: 'a + Clone>(
    state: &'a mut button::State,
    show_password: bool,
    on_press: Message,
    theme: &dyn Theme,
) -> Element<'a, Message> {
    if show_password {
        icon_button(
            ButtonData {
                state,
                icon: Icon::EyeSlash,
                text: "Hide",
                kind: ButtonKind::Normal,
                on_press: Some(on_press),
            },
            "Hide password",
            true,
            theme,
        )
    } else {
        icon_button(
            ButtonData {
                state,
                icon: Icon::Eye,
                text: "Show",
                kind: ButtonKind::Normal,
                on_press: Some(on_press),
            },
            "Hide password",
            true,
            theme,
        )
    }
}

/// Create a default [`TextInput`](TextInput).
///
/// It expects:
///  - The [`State`](text_input::State) of the [`TextInput`](TextInput)
///  - The placeholder of the [`TextInput`](TextInput)
///  - The value of the [`TextInput`](TextInput)
///  - The message that the [`TextInput`](TextInput) sends if the value is changed
#[cfg_attr(coverage, no_coverage)]
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
#[cfg_attr(coverage, no_coverage)]
pub fn centered_container_with_column<'a, Message: 'a>(
    children: Vec<Element<'a, Message>>,
    theme: &dyn Theme,
) -> Container<'a, Message> {
    Container::new(
        Column::with_children(children)
            .max_width(DEFAULT_MAX_WIDTH)
            .padding(DEFAULT_COLUMN_PADDING)
            .spacing(DEFAULT_COLUMN_SPACING),
    )
    .style(theme.container())
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
}

/// Create a default vertical [`Space`](Space).
#[cfg_attr(coverage, no_coverage)]
pub fn default_vertical_space() -> Space {
    vertical_space(1)
}

/// Create a vertical [`Space`] that is `factor` times larger than the default [`Space`](Space).
#[cfg_attr(coverage, no_coverage)]
pub fn vertical_space(factor: u16) -> Space {
    Space::with_height(Length::Units(factor * DEFAULT_SPACE_HEIGHT))
}

/// Calculate the strength of the given password.
#[allow(clippy::unused_async)]
pub async fn estimate_password_strength(
    password: pwduck_core::SecString,
) -> Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError> {
    Ok(pwduck_core::password_entropy(&password))
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

#[cfg(test)]
mod tests {
    use super::SomeIf;

    #[test]
    fn some_if() {
        struct Test;
        impl SomeIf for Test {}

        assert!(Test.some_if(true).is_some());
        assert!(Test.some_if(false).is_none());
    }

    #[test]
    fn some_if_not() {
        struct Test;
        impl SomeIf for Test {}

        assert!(Test.some_if_not(false).is_some());
        assert!(Test.some_if_not(true).is_none());
    }
}
