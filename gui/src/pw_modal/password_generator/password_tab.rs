//! The state of the password generator.
use std::ops::RangeInclusive;

use iced::{
    button, slider, Button, Column, Command, Container, Element, Length, Row, Slider, Text,
};
use iced_aw::{number_input, NumberInput};

use crate::{theme::Theme, utils::vertical_space, DEFAULT_COLUMN_SPACING, DEFAULT_ROW_SPACING};

use bitflags::bitflags;

#[cfg(test)]
use mocktopus::macros::*;

/// The default length of the password tab.
const DEFAULT_LENGTH: u8 = 32;

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

#[cfg_attr(test, mockable)]
impl PasswordTabState {
    /// Create a new [`PasswordTabState`](PasswordTabState).
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            ..Self::default()
        }
    }

    /// Update the length of the password to the given value.
    fn update_length(&mut self, length: u8) -> Command<PasswordTabMessage> {
        self.length = length;
        Command::none()
    }

    /// Toggle the inclusion of upper symbols.
    fn toggle_upper_symbols(&mut self) -> Command<PasswordTabMessage> {
        self.flags.toggle(Flags::INCLUDE_UPPER);
        Command::none()
    }

    /// Toggle the inclusion of lower symbols.
    fn toggle_lower_symbols(&mut self) -> Command<PasswordTabMessage> {
        self.flags.toggle(Flags::INCLUDE_LOWER);
        Command::none()
    }

    /// Toggle the inclusion of number symbols.
    fn toggle_numbers_symbols(&mut self) -> Command<PasswordTabMessage> {
        self.flags.toggle(Flags::INCLUDE_NUMBERS);
        Command::none()
    }

    /// Toggle the inclusion of special symbols.
    fn toggle_special_symbols(&mut self) -> Command<PasswordTabMessage> {
        self.flags.toggle(Flags::INCLUDE_SPECIAL);
        Command::none()
    }

    /// Update the [`PasswordTabState`](PasswordTabState) with the given message.
    pub fn update(&mut self, message: &PasswordTabMessage) -> Command<PasswordTabMessage> {
        match message {
            PasswordTabMessage::LengthSlider(length) | PasswordTabMessage::LengthInput(length) => {
                self.update_length(*length)
            }
            PasswordTabMessage::IncludeUpper => self.toggle_upper_symbols(),
            PasswordTabMessage::IncludeLower => self.toggle_lower_symbols(),
            PasswordTabMessage::IncludeNumbers => self.toggle_numbers_symbols(),
            PasswordTabMessage::IncludeSpecial => self.toggle_special_symbols(),
        }
    }

    /// Create the view of the [`PasswordTabState`](PasswordTabState).
    #[cfg_attr(coverage, no_coverage)]
    pub fn view(&mut self, theme: &dyn Theme) -> Element<PasswordTabMessage> {
        let length = Text::new("Length:");

        let length_slider = Slider::new(
            &mut self.length_slider_state,
            RangeInclusive::new(0, 128),
            self.length,
            PasswordTabMessage::LengthSlider,
        )
        .style(theme.slider());

        let length_number_input = NumberInput::new(
            &mut self.length_input_state,
            self.length,
            128,
            PasswordTabMessage::LengthInput,
        )
        .style(theme.number_input())
        .input_style(theme.text_input());

        let length_row = Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .align_items(iced::Align::Center)
            .push(length)
            .push(length_slider)
            .push(length_number_input);

        let characters = Text::new("Include characters:");

        let mut include_upper = Button::new(&mut self.include_upper_state, Text::new("A-Z"))
            .on_press(PasswordTabMessage::IncludeUpper);
        include_upper = include_upper.style(if self.flags.contains(Flags::INCLUDE_UPPER) {
            theme.toggle_button_active()
        } else {
            theme.toggle_button_inactive()
        });

        let mut include_lower = Button::new(&mut self.include_lower_state, Text::new("a-z"))
            .on_press(PasswordTabMessage::IncludeLower);
        include_lower = include_lower.style(if self.flags.contains(Flags::INCLUDE_LOWER) {
            theme.toggle_button_active()
        } else {
            theme.toggle_button_inactive()
        });

        let mut include_numbers = Button::new(&mut self.include_numbers_state, Text::new("0-9"))
            .on_press(PasswordTabMessage::IncludeNumbers);
        include_numbers = include_numbers.style(if self.flags.contains(Flags::INCLUDE_NUMBERS) {
            theme.toggle_button_active()
        } else {
            theme.toggle_button_inactive()
        });

        let mut include_special =
            Button::new(&mut self.include_special_state, Text::new("&?!*..."))
                .on_press(PasswordTabMessage::IncludeSpecial);
        include_special = include_special.style(if self.flags.contains(Flags::INCLUDE_SPECIAL) {
            theme.toggle_button_active()
        } else {
            theme.toggle_button_inactive()
        });

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

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
    };

    use iced::Command;
    use mocktopus::mocking::*;

    use super::{Flags, PasswordTabMessage, PasswordTabState, DEFAULT_LENGTH};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    #[test]
    fn new() {
        let state = PasswordTabState::new();

        assert_eq!(state.length, DEFAULT_LENGTH);

        assert!(state.flags.is_all());
    }

    #[test]
    fn update_length() {
        let mut state = PasswordTabState::new();

        assert_eq!(state.length, DEFAULT_LENGTH);

        state.update_length(128);

        assert_eq!(state.length, 128);
    }

    #[test]
    fn toggle_upper_symbols() {
        let mut state = PasswordTabState::new();

        assert!(state.flags.contains(Flags::INCLUDE_UPPER));

        state.toggle_upper_symbols();

        assert!(!state.flags.contains(Flags::INCLUDE_UPPER));

        state.toggle_upper_symbols();

        assert!(state.flags.contains(Flags::INCLUDE_UPPER));
    }

    #[test]
    fn toggle_lower_symbols() {
        let mut state = PasswordTabState::new();

        assert!(state.flags.contains(Flags::INCLUDE_LOWER));

        state.toggle_lower_symbols();

        assert!(!state.flags.contains(Flags::INCLUDE_LOWER));

        state.toggle_lower_symbols();

        assert!(state.flags.contains(Flags::INCLUDE_LOWER));
    }

    #[test]
    fn toggle_numbers_symbols() {
        let mut state = PasswordTabState::new();

        assert!(state.flags.contains(Flags::INCLUDE_NUMBERS));

        state.toggle_numbers_symbols();

        assert!(!state.flags.contains(Flags::INCLUDE_NUMBERS));

        state.toggle_numbers_symbols();

        assert!(state.flags.contains(Flags::INCLUDE_NUMBERS));
    }

    #[test]
    fn toggle_special_symbols() {
        let mut state = PasswordTabState::new();

        assert!(state.flags.contains(Flags::INCLUDE_SPECIAL));

        state.toggle_special_symbols();

        assert!(!state.flags.contains(Flags::INCLUDE_SPECIAL));

        state.toggle_special_symbols();

        assert!(state.flags.contains(Flags::INCLUDE_SPECIAL));
    }

    #[test]
    fn update() {
        let mut state = PasswordTabState::new();

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(PasswordTabState::update_length.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordTabState::toggle_upper_symbols.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordTabState::toggle_lower_symbols.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordTabState::toggle_numbers_symbols.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordTabState::toggle_special_symbols.type_id(), 0);

            PasswordTabState::update_length.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordTabState::update_length.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordTabState::toggle_upper_symbols.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordTabState::toggle_upper_symbols.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordTabState::toggle_lower_symbols.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordTabState::toggle_lower_symbols.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordTabState::toggle_numbers_symbols.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordTabState::toggle_numbers_symbols.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordTabState::toggle_special_symbols.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordTabState::toggle_special_symbols.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Update length
            assert_eq!(
                call_map.borrow()[&PasswordTabState::update_length.type_id()],
                0
            );
            let _ = state.update(&PasswordTabMessage::LengthSlider(42));
            assert_eq!(
                call_map.borrow()[&PasswordTabState::update_length.type_id()],
                1
            );
            let _ = state.update(&PasswordTabMessage::LengthInput(42));
            assert_eq!(
                call_map.borrow()[&PasswordTabState::update_length.type_id()],
                2
            );

            // Toggle upper symbols
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_upper_symbols.type_id()],
                0
            );
            let _ = state.update(&PasswordTabMessage::IncludeUpper);
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_upper_symbols.type_id()],
                1
            );

            // Toggle lower symbols
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_lower_symbols.type_id()],
                0
            );
            let _ = state.update(&PasswordTabMessage::IncludeLower);
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_lower_symbols.type_id()],
                1
            );

            // Toggle numbers symbols
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_numbers_symbols.type_id()],
                0
            );
            let _ = state.update(&PasswordTabMessage::IncludeNumbers);
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_numbers_symbols.type_id()],
                1
            );

            // Toggle special symbols
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_special_symbols.type_id()],
                0
            );
            let _ = state.update(&PasswordTabMessage::IncludeSpecial);
            assert_eq!(
                call_map.borrow()[&PasswordTabState::toggle_special_symbols.type_id()],
                1
            );
        });
    }

    #[test]
    fn default_flags() {
        let flags = Flags::default();

        assert!(flags.is_all());
    }
}
