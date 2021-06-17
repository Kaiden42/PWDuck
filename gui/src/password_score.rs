//! TODO

use iced::{container, Column, Container, Element, Length, Row, Text};

use crate::{utils::default_vertical_space, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING};

/// TODO
#[derive(Debug)]
pub struct PasswordScore {
    /// TODO
    password_info: Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError>,
}
impl PasswordScore {
    /// TODO
    pub const fn new(
        password_info: Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError>,
    ) -> Self {
        Self { password_info }
    }

    /// TODO
    pub fn view<'a, Message: 'a + Clone>(&'a mut self) -> Element<Message> {
        match self.password_info.as_ref() {
            Ok(password_info) => {
                let entropy = password_info.get_entropy();
                let password_strength = PasswordStrength::from_entropy(entropy);

                let strength_bar = create_strength_bar(password_strength);

                Column::new()
                    .spacing(DEFAULT_COLUMN_SPACING)
                    .push(strength_bar)
                    .push(
                        Row::new()
                            .spacing(DEFAULT_ROW_SPACING)
                            .width(Length::Fill)
                            .push(
                                Text::new(format!("Strength: {}", password_strength))
                                    .width(Length::Fill)
                                    .size(16),
                            )
                            .push(
                                Text::new(format!("Entropy: {:.5} bits", entropy))
                                    .width(Length::Fill)
                                    .size(16)
                                    .horizontal_alignment(iced::HorizontalAlignment::Right),
                            ),
                    )
                    .into()
            }
            Err(_) => Text::new("Error").into(),
        }
    }
}

/// TODO
fn create_strength_bar<'a, Message>(password_strength: PasswordStrength) -> Row<'a, Message>
where
    Message: 'a + Clone,
{
    let num_of_bars = 5;
    let strength_bar = Row::new().width(Length::Fill);

    match password_strength {
        PasswordStrength::Bad => fill_strength_bar(strength_bar, 1, num_of_bars - 1, BadStyle),
        PasswordStrength::Weak => fill_strength_bar(strength_bar, 2, num_of_bars - 2, WeakStyle),
        PasswordStrength::Good => fill_strength_bar(strength_bar, 3, num_of_bars - 3, GoodStyle),
        PasswordStrength::Strong => {
            fill_strength_bar(strength_bar, 4, num_of_bars - 4, StrongStyle)
        }
        PasswordStrength::Awesome => {
            fill_strength_bar(strength_bar, 5, num_of_bars - 5, AwesomeStyle)
        }
    }
}

/// TODO
fn fill_strength_bar<'a, Style: 'static, Message>(
    mut strength_bar: Row<'a, Message>,
    colored: usize,
    uncolored: usize,
    style: Style,
) -> Row<'a, Message>
where
    Style: Copy + container::StyleSheet,
    Message: 'a + Clone,
{
    for _ in 0..colored {
        strength_bar = strength_bar.push(
            Container::new(default_vertical_space())
                .style(style)
                .width(Length::Fill),
        )
    }

    for _ in 0..uncolored {
        strength_bar = strength_bar.push(
            Container::new(default_vertical_space())
                .style(ClearStyle)
                .width(Length::Fill),
        )
    }

    strength_bar
}

/// TODO
#[derive(Copy, Clone, Debug, Default)]
struct ClearStyle;

impl container::StyleSheet for ClearStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: None,
            border_radius: 0.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
        }
    }
}

/// TODO
#[derive(Copy, Clone, Debug, Default)]
struct BadStyle;

impl container::StyleSheet for BadStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(iced::Color::from_rgb(1.0, 0.0, 0.0).into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
        }
    }
}

/// TODO
#[derive(Copy, Clone, Debug, Default)]
struct WeakStyle;

impl container::StyleSheet for WeakStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(iced::Color::from_rgb(1.0, 0.65, 0.0).into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
        }
    }
}

/// TODO
#[derive(Copy, Clone, Debug, Default)]
struct GoodStyle;

impl container::StyleSheet for GoodStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(iced::Color::from_rgb(1.0, 1.0, 0.0).into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
        }
    }
}

/// TODO
#[derive(Copy, Clone, Debug, Default)]
struct StrongStyle;

impl container::StyleSheet for StrongStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(iced::Color::from_rgb(0.0, 1.0, 0.0).into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
        }
    }
}

/// TODO
#[derive(Copy, Clone, Debug, Default)]
struct AwesomeStyle;

impl container::StyleSheet for AwesomeStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: None,
            background: Some(iced::Color::from_rgb(0.0, 0.0, 1.0).into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: iced::Color::BLACK,
        }
    }
}

/// TODO
#[derive(Clone, Copy, Debug)]
enum PasswordStrength {
    /// TODO
    Bad,
    /// TODO
    Weak,
    /// TODO
    Good,
    /// TODO
    Strong,
    /// TODO
    Awesome,
}

impl PasswordStrength {
    /// TODO
    const fn from_entropy(entropy: f64) -> Self {
        match entropy as u32 {
            0..=50 => Self::Bad,
            51..=100 => Self::Weak,
            101..=200 => Self::Good,
            201..=300 => Self::Strong,
            _ => Self::Awesome,
        }
    }
}

impl std::fmt::Display for PasswordStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PasswordStrength::Bad => "Bad",
                PasswordStrength::Weak => "Weak",
                PasswordStrength::Good => "Good",
                PasswordStrength::Strong => "Strong",
                PasswordStrength::Awesome => "Awesome",
            }
        )
    }
}
