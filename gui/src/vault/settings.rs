//! TODO

use iced::{Column, Command, Radio, Text};

use crate::Component;
use crate::{
    utils::centered_container_with_column, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING,
    DEFAULT_HEADER_SIZE,
};

/// The state of the settings view.
#[derive(Debug)]
pub struct Settings {}

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
        match message {
            SettingsMessage::ThemeChanged(theme) => {
                application_settings.set_theme(theme);
            }
        }
        // TODO: maybe async
        pwduck_core::save_application_settings(application_settings)?;
        Ok(Command::none())
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
