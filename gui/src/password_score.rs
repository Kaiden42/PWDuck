//! Displays the score of the password.

use iced::{container, Column, Container, Element, Length, Row, Text};

use crate::{utils::default_vertical_space, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING};

/// Displays the score of the password.
#[derive(Debug)]
pub struct PasswordScore {
    /// The info about the password.
    password_info: Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError>,
}
impl PasswordScore {
    /// Create a new [`PasswordScore`](PasswordScore).
    pub const fn new(
        password_info: Result<pwduck_core::PasswordInfo, pwduck_core::PWDuckCoreError>,
    ) -> Self {
        Self { password_info }
    }

    /// Create the view of this [`PasswordScore`](PasswordScore).
    #[cfg_attr(coverage, no_coverage)]
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

/// Create the view of the strength bar based on the given strength.
#[cfg_attr(coverage, no_coverage)]
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

/// Fill the strength bar.
///
/// It expects:
///  - A mutable reference on the strength bar to fill
///  - The amount of colored parts contained in this bar
///  - The amount of uncolored parts contained in this bar
///  - The style to color the colored parts with
#[cfg_attr(coverage, no_coverage)]
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
        );
    }

    for _ in 0..uncolored {
        strength_bar = strength_bar.push(
            Container::new(default_vertical_space())
                .style(ClearStyle)
                .width(Length::Fill),
        );
    }

    strength_bar
}

/// The style of the uncolored parts of the strength bar.
#[derive(Copy, Clone, Debug, Default)]
struct ClearStyle;

impl container::StyleSheet for ClearStyle {
    #[cfg_attr(coverage, no_coverage)]
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

/// The style of the colored parts of the strength bar if the strength of the password is bad.
#[derive(Copy, Clone, Debug, Default)]
struct BadStyle;

impl container::StyleSheet for BadStyle {
    #[cfg_attr(coverage, no_coverage)]
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

/// The style of the colored parts of the strength bar if the strength of the password is weak.
#[derive(Copy, Clone, Debug, Default)]
struct WeakStyle;

impl container::StyleSheet for WeakStyle {
    #[cfg_attr(coverage, no_coverage)]
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

/// The style of the colored parts of the strength bar if the strength of the password is good.
#[derive(Copy, Clone, Debug, Default)]
struct GoodStyle;

impl container::StyleSheet for GoodStyle {
    #[cfg_attr(coverage, no_coverage)]
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

/// The style of the colored parts of the strength bar if the strength of the password is strong.
#[derive(Copy, Clone, Debug, Default)]
struct StrongStyle;

impl container::StyleSheet for StrongStyle {
    #[cfg_attr(coverage, no_coverage)]
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

/// The style of the colored parts of the strength bar if the strength of the password is awesome.
#[derive(Copy, Clone, Debug, Default)]
struct AwesomeStyle;

impl container::StyleSheet for AwesomeStyle {
    #[cfg_attr(coverage, no_coverage)]
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

/// The strength of the password
#[derive(Clone, Copy, Debug)]
enum PasswordStrength {
    /// The strength of the password is bad.
    Bad,
    /// The strength of the password is weak.
    Weak,
    /// The strength of the password is good.
    Good,
    /// The strength of the password is strong.
    Strong,
    /// The strength of the password is awesome.
    Awesome,
}

impl PasswordStrength {
    /// Calculate the strength based on the given entropy.
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
    #[cfg_attr(coverage, no_coverage)]
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

#[cfg(test)]
mod tests {
    use super::PasswordStrength;

    #[test]
    fn from_entropy() {
        // Bad
        if let PasswordStrength::Bad = PasswordStrength::from_entropy(0.0) {
        } else {
            panic!("Password strength should be bad");
        }
        if let PasswordStrength::Bad = PasswordStrength::from_entropy(42.0) {
        } else {
            panic!("Password strength should be bad");
        }
        if let PasswordStrength::Bad = PasswordStrength::from_entropy(50.0) {
        } else {
            panic!("Password strength should be bad");
        }

        // Weak
        if let PasswordStrength::Weak = PasswordStrength::from_entropy(51.0) {
        } else {
            panic!("Password strength should be weak");
        }
        if let PasswordStrength::Weak = PasswordStrength::from_entropy(84.0) {
        } else {
            panic!("Password strength should be weak");
        }
        if let PasswordStrength::Weak = PasswordStrength::from_entropy(100.0) {
        } else {
            panic!("Password strength should be weak");
        }

        // Good
        if let PasswordStrength::Good = PasswordStrength::from_entropy(101.0) {
        } else {
            panic!("Password strength should be good");
        }
        if let PasswordStrength::Good = PasswordStrength::from_entropy(142.0) {
        } else {
            panic!("Password strength should be good");
        }
        if let PasswordStrength::Good = PasswordStrength::from_entropy(200.0) {
        } else {
            panic!("Password strength should be good");
        }

        // Strong
        if let PasswordStrength::Strong = PasswordStrength::from_entropy(201.0) {
        } else {
            panic!("Password strength should be strong");
        }
        if let PasswordStrength::Strong = PasswordStrength::from_entropy(242.0) {
        } else {
            panic!("Password strength should be strong");
        }
        if let PasswordStrength::Strong = PasswordStrength::from_entropy(300.0) {
        } else {
            panic!("Password strength should be strong");
        }

        // Awesome
        if let PasswordStrength::Awesome = PasswordStrength::from_entropy(301.0) {
        } else {
            panic!("Password strength should be awesome");
        }
        if let PasswordStrength::Awesome = PasswordStrength::from_entropy(1000.0) {
        } else {
            panic!("Password strength should be awesome");
        }
    }
}
