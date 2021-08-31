//! TODO

use iced::{Column, Command, Radio, Text};
use iced_focus::Focus;
use pwduck_core::ApplicationSettings;

use crate::error::PWDuckGuiError;
use crate::Component;
use crate::{
    utils::centered_container_with_column, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING,
    DEFAULT_HEADER_SIZE,
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the settings view.
#[derive(Debug, Focus)]
pub struct Settings {}

#[cfg_attr(test, mockable)]
impl Settings {
    /// Update the application settings's theme and replace it with the given value.
    #[allow(clippy::unused_self)]
    fn update_theme(
        &mut self,
        application_settings: &mut ApplicationSettings,
        theme: pwduck_core::theme::Theme,
    ) -> Command<SettingsMessage> {
        application_settings.set_theme(theme);
        Command::none()
    }

    /// Save the application settings to disk.
    // TODO: maybe async
    #[allow(clippy::unused_self)]
    fn save_application_settings(
        &mut self,
        application_settings: &ApplicationSettings,
    ) -> Result<(), PWDuckGuiError> {
        pwduck_core::save_application_settings(application_settings)?;
        Ok(())
    }
}

/// The message produced by the settings view.
#[derive(Clone, Debug)]
pub enum SettingsMessage {
    /// Change the theme to the new value.
    ThemeChanged(pwduck_core::theme::Theme),
}

impl Component for Settings {
    type Message = SettingsMessage;
    type ConstructorParam = ();

    fn new(_: Self::ConstructorParam) -> Self {
        Self {}
    }

    fn title(&self) -> String {
        "Settings".into()
    }

    fn update<P: crate::Platform + 'static>(
        &mut self,
        message: Self::Message,
        application_settings: &mut pwduck_core::ApplicationSettings,
        _modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        _clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, crate::error::PWDuckGuiError> {
        let cmd = match message {
            SettingsMessage::ThemeChanged(theme) => self.update_theme(application_settings, theme),
        };
        self.save_application_settings(application_settings)?;
        Ok(cmd)
    }

    fn view<P: crate::Platform + 'static>(
        &mut self,
        application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn crate::theme::Theme,
        _viewport: &crate::Viewport,
    ) -> iced::Element<'_, Self::Message> {
        let theme_column = Column::new()
            .spacing(DEFAULT_COLUMN_SPACING)
            .padding(DEFAULT_COLUMN_PADDING)
            .push(Text::new("Theme:").size(DEFAULT_HEADER_SIZE))
            .push(Radio::new(
                pwduck_core::theme::Theme::Light,
                "Light",
                Some(*application_settings.theme()),
                SettingsMessage::ThemeChanged,
            ))
            .push(Radio::new(
                pwduck_core::theme::Theme::Dark,
                "Dark",
                Some(*application_settings.theme()),
                SettingsMessage::ThemeChanged,
            ));

        centered_container_with_column(vec![theme_column.into()], theme).into()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use iced::Command;
    use mocktopus::mocking::*;

    use crate::{Component, TestPlatform};

    use super::{Settings, SettingsMessage};

    thread_local! {
        static CALL_MAP: RefCell<HashMap<String, usize>> = RefCell::new(HashMap::new());
    }

    const UPDATE_THEME: &str = "update_theme";
    const SAVE_APPLICATION_SETTINGS: &str = "save_application_settings";

    #[test]
    fn update_theme() {
        let mut settings = Settings::new(());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        assert_eq!(
            application_settings.theme(),
            &pwduck_core::theme::Theme::Light
        );

        let _cmd =
            settings.update_theme(&mut application_settings, pwduck_core::theme::Theme::Dark);

        assert_eq!(
            application_settings.theme(),
            &pwduck_core::theme::Theme::Dark
        );
    }

    #[test]
    fn update() {
        let mut settings = Settings::new(());
        let mut application_settings = pwduck_core::ApplicationSettings::default();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::default());
        // WARNING: This is highly unsafe!
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map.borrow_mut().insert(UPDATE_THEME.to_owned(), 0);
            call_map
                .borrow_mut()
                .insert(SAVE_APPLICATION_SETTINGS.to_owned(), 0);

            Settings::update_theme.mock_raw(|_self, _settings, _theme| {
                call_map.borrow_mut().get_mut(UPDATE_THEME).map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            Settings::save_application_settings.mock_raw(|_self, _settings| {
                call_map
                    .borrow_mut()
                    .get_mut(SAVE_APPLICATION_SETTINGS)
                    .map(|c| *c += 1);
                MockResult::Return(Ok(()))
            });

            // Update theme
            assert_eq!(call_map.borrow()[UPDATE_THEME], 0);
            assert_eq!(call_map.borrow()[SAVE_APPLICATION_SETTINGS], 0);
            let _ = settings.update::<TestPlatform>(
                SettingsMessage::ThemeChanged(pwduck_core::theme::Theme::Dark),
                &mut application_settings,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[UPDATE_THEME], 1);
            assert_eq!(call_map.borrow()[SAVE_APPLICATION_SETTINGS], 1);

            assert!(call_map
                .borrow()
                .iter()
                .filter(|(k, _)| k.as_str() != SAVE_APPLICATION_SETTINGS)
                .all(|(_, v)| *v == 1));
            assert_eq!(call_map.borrow()[SAVE_APPLICATION_SETTINGS], 1);
        })
    }
}
