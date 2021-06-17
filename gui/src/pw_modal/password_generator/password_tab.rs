//! TODO

use std::ops::RangeInclusive;

use iced::{
    button, slider, Button, Column, Command, Container, Element, Length, Row, Slider, Text,
};
use iced_aw::{number_input, NumberInput};

use crate::{
    error::PWDuckGuiError, utils::vertical_space, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING,
};

/// TODO
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct PasswordTabState {
    /// TODO
    length: u8,
    /// TODO
    length_slider_state: slider::State,
    /// TODO
    length_input_state: number_input::State,

    /// TODO
    include_upper: bool,
    /// TODO
    include_upper_state: button::State,

    /// TODO
    include_lower: bool,
    /// TODO
    include_lower_state: button::State,

    /// TODO
    include_numbers: bool,
    /// TODO
    include_numbers_state: button::State,

    /// TODO
    include_special: bool,
    /// TODO
    include_special_state: button::State,
}

/// TODO
#[derive(Clone, Debug)]
pub enum PasswordTabMessage {
    /// TODO
    LengthSlider(u8),
    /// TODO
    LengthInput(u8),
    /// TODO
    IncludeUpper,
    /// TODO
    IncludeLower,
    /// TODO
    IncludeNumbers,
    /// TODO
    IncludeSpecial,
}

impl PasswordTabState {
    /// TODO
    pub fn new() -> Self {
        Self {
            length: 32,
            include_upper: true,
            include_lower: true,
            include_numbers: true,
            include_special: true,
            ..Self::default()
        }
    }

    /// TODO
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
                self.include_upper = !self.include_upper;
                Command::none()
            }
            PasswordTabMessage::IncludeLower => {
                self.include_lower = !self.include_lower;
                Command::none()
            }
            PasswordTabMessage::IncludeNumbers => {
                self.include_numbers = !self.include_numbers;
                Command::none()
            }
            PasswordTabMessage::IncludeSpecial => {
                self.include_special = !self.include_special;
                Command::none()
            }
        };

        Ok(cmd)
    }

    /// TODO
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
        if self.include_upper {
            include_upper = include_upper.style(ActivatedButtonStyle);
        }

        let mut include_lower = Button::new(&mut self.include_lower_state, Text::new("a-z"))
            .on_press(PasswordTabMessage::IncludeLower);
        if self.include_lower {
            include_lower = include_lower.style(ActivatedButtonStyle);
        }

        let mut include_numbers = Button::new(&mut self.include_numbers_state, Text::new("0-9"))
            .on_press(PasswordTabMessage::IncludeNumbers);
        if self.include_numbers {
            include_numbers = include_numbers.style(ActivatedButtonStyle)
        }

        let mut include_special =
            Button::new(&mut self.include_special_state, Text::new("&?!*..."))
                .on_press(PasswordTabMessage::IncludeSpecial);
        if self.include_special {
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

    /// TODO
    pub fn generate(&self) -> String {
        use pwduck_core::Symbols;

        let mut symbols = Symbols::new();

        if self.include_upper {
            symbols.append(&Symbols::UPPER_ALPHA);
        }
        if self.include_lower {
            symbols.append(&Symbols::LOWER_ALPHA);
        }
        if self.include_numbers {
            symbols.append(&Symbols::NUMBERS);
        }
        if self.include_special {
            symbols.append(&Symbols::SPECIAL);
        }

        pwduck_core::generate_password(self.length, &symbols).unwrap()
    }
}

/// TODO
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
