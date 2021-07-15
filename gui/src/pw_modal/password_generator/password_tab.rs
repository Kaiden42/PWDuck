//! TODO

use std::ops::RangeInclusive;

use iced::{
    button, slider, Button, Column, Command, Container, Element, Length, Row, Slider, Text,
};
use iced_aw::{number_input, NumberInput};

use crate::{
    error::PWDuckGuiError, utils::vertical_space, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING,
};

use bitflags::bitflags;

/// The state of the password generator tab.
#[derive(Debug, Default)]
pub struct PasswordTabState {
    /// The length of the password to generate.
    length: u8,
    /// The state of the [`Slider`](Slider) to set the length.
    length_slider_state: slider::State,
    /// The state of the [`NumberInput`](NumberInput) to set the length.
    length_input_state: number_input::State,

    /// The state of the [`Button`](Button) to toggle the inclusion of upper case characters.
    include_upper_state: button::State,

    /// The state of the [`Button`](Button) to toggle the inclusion of lower case characters.
    include_lower_state: button::State,

    /// The state of the [`Button`](Button) to toggle the inclusion of digits.
    include_numbers_state: button::State,

    /// The state of the [`Button`](Button) to toggle the inclusion of special characters.
    include_special_state: button::State,

    /// The configuration of this [`PasswordTabState`](PasswordTabState).
    flags: Flags,
}

/// The message produced by the password generator tab.
#[derive(Clone, Debug)]
pub enum PasswordTabMessage {
    /// The length slider modified the length's value.
    LengthSlider(u8),
    /// The length number input modified the length's value.
    LengthInput(u8),
    /// Toggle the inclusion of upper case latin characters.
    IncludeUpper,
    /// Toggle the inclusion of lower case latin characters.
    IncludeLower,
    /// Toggle the inclusion of digits.
    IncludeNumbers,
    /// Toggle the inclusion of special characters.
    IncludeSpecial,
}

impl PasswordTabState {
    /// Create a new [`PasswordTabState`](PasswordTabState).
    pub fn new() -> Self {
        Self {
            length: 32,
            ..Self::default()
        }
    }

    /// Update the [`PasswordTabState`](PasswordTabState) with the given message.
    pub fn update(
        &mut self,
        message: &PasswordTabMessage,
    ) -> Result<Command<PasswordTabMessage>, PWDuckGuiError> {
        let cmd = match message {
            PasswordTabMessage::LengthSlider(length) | PasswordTabMessage::LengthInput(length) => {
                self.length = *length;
                Command::none()
            }
            PasswordTabMessage::IncludeUpper => {
                self.flags.toggle(Flags::INCLUDE_UPPER);
                Command::none()
            }
            PasswordTabMessage::IncludeLower => {
                self.flags.toggle(Flags::INCLUDE_LOWER);
                Command::none()
            }
            PasswordTabMessage::IncludeNumbers => {
                self.flags.toggle(Flags::INCLUDE_NUMBERS);
                Command::none()
            }
            PasswordTabMessage::IncludeSpecial => {
                self.flags.toggle(Flags::INCLUDE_SPECIAL);
                Command::none()
            }
        };

        Ok(cmd)
    }

    /// Create the view of the [`PasswordTabState`](PasswordTabState).
    pub fn view(&mut self) -> Element<PasswordTabMessage> {
        let length = Text::new("Length:");

        let length_slider = Slider::new(
            &mut self.length_slider_state,
            RangeInclusive::new(0, 128),
            self.length,
            PasswordTabMessage::LengthSlider,
        );

        let length_number_input = NumberInput::new(
            &mut self.length_input_state,
            self.length,
            128,
            PasswordTabMessage::LengthInput,
        );

        let length_row = Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .align_items(iced::Align::Center)
            .push(length)
            .push(length_slider)
            .push(length_number_input);

        let characters = Text::new("Include characters:");

        let mut include_upper = Button::new(&mut self.include_upper_state, Text::new("A-Z"))
            .on_press(PasswordTabMessage::IncludeUpper);
        if self.flags.contains(Flags::INCLUDE_UPPER) {
            include_upper = include_upper.style(ActivatedButtonStyle);
        }

        let mut include_lower = Button::new(&mut self.include_lower_state, Text::new("a-z"))
            .on_press(PasswordTabMessage::IncludeLower);
        if self.flags.contains(Flags::INCLUDE_LOWER) {
            include_lower = include_lower.style(ActivatedButtonStyle);
        }

        let mut include_numbers = Button::new(&mut self.include_numbers_state, Text::new("0-9"))
            .on_press(PasswordTabMessage::IncludeNumbers);
        if self.flags.contains(Flags::INCLUDE_NUMBERS) {
            include_numbers = include_numbers.style(ActivatedButtonStyle)
        }

        let mut include_special =
            Button::new(&mut self.include_special_state, Text::new("&?!*..."))
                .on_press(PasswordTabMessage::IncludeSpecial);
        if self.flags.contains(Flags::INCLUDE_SPECIAL) {
            include_special = include_special.style(ActivatedButtonStyle)
        }

        let character_container = Container::new(
            Row::new()
                .spacing(2 * DEFAULT_ROW_SPACING)
                .push(include_upper)
                .push(include_lower)
                .push(include_numbers)
                .push(include_special),
        )
        .width(Length::Fill)
        .center_x()
        .center_y();

        Column::new()
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(length_row)
            .push(vertical_space(2))
            .push(characters)
            .push(character_container)
            .into()
    }

    /// Generate a new random password.
    pub fn generate(&self) -> String {
        use pwduck_core::Symbols;

        let mut symbols = Symbols::new();

        if self.flags.contains(Flags::INCLUDE_UPPER) {
            symbols.append(&Symbols::UPPER_ALPHA);
        }
        if self.flags.contains(Flags::INCLUDE_LOWER) {
            symbols.append(&Symbols::LOWER_ALPHA);
        }
        if self.flags.contains(Flags::INCLUDE_NUMBERS) {
            symbols.append(&Symbols::NUMBERS);
        }
        if self.flags.contains(Flags::INCLUDE_SPECIAL) {
            symbols.append(&Symbols::SPECIAL);
        }

        pwduck_core::generate_password(self.length, &symbols).unwrap()
    }
}

bitflags! {
    /// The configuration of the [`PasswordTabState`](PasswordTabState).
    pub struct Flags: u8 {
        /// If the generator has to include upper case latin characters to the pool.
        const INCLUDE_UPPER = 0b1;
        /// If the generator has to include lower case latin characters to the pool.
        const INCLUDE_LOWER = 0b1 << 1;
        /// If the generator has to include digitals (`0-9`) to the pool.
        const INCLUDE_NUMBERS = 0b1 << 2;
        /// If the generator has to include special characters (`\/{>():...`) to the pool.
        const INCLUDE_SPECIAL = 0b1 << 3;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::all()
    }
}

/// The style of the toggle buttons.
#[derive(Debug, Default)]
struct ActivatedButtonStyle;

impl button::StyleSheet for ActivatedButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: iced::Vector::default(),
            background: Some(iced::Color::from_rgb(0.2, 0.7, 0.2).into()),
            border_color: iced::Color::BLACK,
            ..button::Style::default()
        }
    }
}
