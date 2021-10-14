//! TODO
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
//#![deny(unused_results)]
#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(coverage, feature(no_coverage))]
#![warn(
    clippy::pedantic,
    clippy::nursery,

    // Restriction lints
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::get_unwrap,
    clippy::let_underscore_must_use,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_docs_in_private_items,
    clippy::multiple_inherent_impl,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::str_to_string,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::unwrap_in_result,
    //clippy::unwrap_used,
    clippy::use_debug,
)]
#![allow(
    clippy::suboptimal_flops,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions,

    // TODO: remove
    clippy::missing_errors_doc,
)]

use std::{marker::PhantomData, path::PathBuf};

use async_trait::async_trait;
use iced::{
    button, executor, tooltip, Application, Button, Column, Command, Container, Element, Length,
    Row, Settings, Subscription, Text, Tooltip,
};

pub mod error;
use error::{NfdError, PWDuckGuiError};

pub mod vault;
use iced_aw::{modal, Card, Modal, TabBar, TabLabel};
use iced_focus::Focus;
use icons::{Icon, ICON_FONT};
use lazy_static::lazy_static;
use pwduck_core::MemKey;
use theme::Theme;
use vault::{
    container::ModifyEntryMessage,
    tab::{VaultContainerMessage, VaultTab},
    tab::{VaultTabMessage, VaultTabVec},
};

mod pw_modal;
mod theme;
mod utils;

use pw_modal::{PasswordGeneratorMessage, PasswordGeneratorState};

use crate::vault::container::ModifyGroupMessage;

mod password_score;

mod icons;

pub use pwduck_core::{Key, Part, Sequence};

/// The default maximum width of a [`Container`](iced::Container).
const DEFAULT_MAX_WIDTH: u32 = 600;
/// The default padding of a [`Column`](iced::Column).
const DEFAULT_COLUMN_PADDING: u16 = 16;
/// The default spacing of a [`Column`](iced::Column).
const DEFAULT_COLUMN_SPACING: u16 = 5;
/// The default spacing of a [`Row`](iced::Row).
const DEFAULT_ROW_SPACING: u16 = 5;
/// The default padding of a [`TextInput`](iced::TextInput).
const DEFAULT_TEXT_INPUT_PADDING: u16 = 5;
/// The default height fo a [`Space`](iced::Space).
const DEFAULT_SPACE_HEIGHT: u16 = 5;
/// The default font size of a header [`Text`](Text).
const DEFAULT_HEADER_SIZE: u16 = 25;
/// The height of the top row.
const TOP_ROW_HEIGHT: u16 = 30;
/// The font size of the top row.
const TOP_ROW_FONT_SIZE: u16 = TOP_ROW_HEIGHT - 10;
/// The padding of the top row.
const TOP_ROW_PADDING: u16 = 5;

lazy_static! {
    //static ref MEM_KEY: Mutex<pwduck_core::MemKey> = Mutex::new(MemKey::new());
    static ref MEM_KEY: std::sync::Mutex<pwduck_core::MemKey> = std::sync::Mutex::new(MemKey::new());

    //static ref MEM_KEY: Arc<RwLock<pwduck_core::MemKey>> = Arc::new(RwLock::new(MemKey::new()));
}

/// The state of the GUI.
#[derive(Debug, Focus)]
pub struct PWDuckGui<P: Platform + 'static> {
    /// The state of the modal
    modal_state: modal::State<ModalState>,
    /// The tabs of open vaults.
    #[focus(enable)]
    //tabs: Vec<VaultTab>,
    tabs: VaultTabVec,
    /// The index of the currently selected tab.
    //selected_tab: usize,

    /// The state of the settings [`Button`](iced::Button).
    settings_state: button::State,
    /// The state of the [`Button`](iced::Button) to open up a new tab.
    open_new_tab_state: button::State,

    /// The size of the window.
    window_size: Viewport,
    /// If the application can exit.
    can_exit: bool,

    /// PhantomData for the [`Platform`](Platform) information.
    phantom: PhantomData<P>,

    /// The settings of this application.
    application_settings: pwduck_core::ApplicationSettings,
}

/// The state of the error dialog.
#[derive(Debug, Default)]
pub struct ErrorDialogState {
    /// The text of the error.
    error: String,
}

impl ErrorDialogState {
    /// Create a new state for the error modal.
    #[cfg_attr(coverage, no_coverage)]
    const fn new(error: String) -> Self {
        Self { error }
    }

    /// Create the view of the error modal.
    #[cfg_attr(coverage, no_coverage)]
    fn view(&mut self, theme: &dyn Theme) -> Element<'_, Message> {
        Card::new(
            Text::new("An error occurred"),
            Text::new(self.error.clone()),
        )
        .max_width(DEFAULT_MAX_WIDTH)
        .on_close(Message::ErrorDialogClose)
        .style(theme.card_warning())
        .into()
    }
}

/// The size of the viewport.
#[derive(Debug, Default)]
pub struct Viewport {
    /// The width of the viewport.
    pub(crate) width: u32,
    /// The height of the viewport.
    pub(crate) height: u32,
}

impl<P: Platform + 'static> PWDuckGui<P> {
    /// Start the gui application.
    #[cfg_attr(coverage, no_coverage)]
    pub fn start() -> Result<(), PWDuckGuiError> {
        pwduck_core::try_to_prevent_core_dump()?;

        let application_settings = pwduck_core::load_application_settings().unwrap_or_default();

        Self::run(Settings {
            exit_on_close_request: false,
            flags: application_settings,
            ..Settings::default()
        })?;
        Ok(())
    }

    /// Update the state of the password generator.
    fn update_password_generator(
        &mut self,
        message: PasswordGeneratorMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Message>, PWDuckGuiError> {
        // TODO: Clean up
        if let ModalState::Password(password_generator_state) = self.modal_state.inner_mut() {
            password_generator_state
                .update(message, clipboard)
                .map(|cmd| cmd.map(Message::PasswordGenerator))
        } else {
            Ok(Command::none())
        }
    }

    /// Hide the error dialog.
    fn close_error_dialog(&mut self) -> iced::Command<Message> {
        // TODO: Clean up
        self.modal_state = modal::State::new(ModalState::None);
        Command::none()
    }

    /// Catch and handle an [`Event`](iced_native::Event) thrown by iced.
    #[cfg_attr(coverage, no_coverage)]
    fn catch_iced_event<Message>(
        &mut self,
        event: iced_native::Event,
    ) -> Result<iced::Command<Message>, PWDuckGuiError> {
        match event {
            iced_native::Event::Window(event) => match event {
                iced_native::window::Event::Resized { width, height } => {
                    self.window_size = Viewport { width, height };
                    Ok(Command::none())
                }
                iced_native::window::Event::CloseRequested => {
                    if self.tabs.iter().any(VaultTab::contains_unsaved_changes) {
                        Err(PWDuckGuiError::VaultContainsUnsavedChanges)
                    } else {
                        self.can_exit = true;
                        Ok(Command::none())
                    }
                }
                _ => Ok(Command::none()),
            },
            _ => Ok(Command::none()),
        }
    }

    /// Update the tab of a vault identified by the given message.
    fn update_vault_tab(
        &mut self,
        index: usize,
        message: VaultTabMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Message>, PWDuckGuiError> {
        self.tabs[index]
            .update::<P>(
                message,
                &mut self.application_settings,
                &mut self.modal_state,
                clipboard,
            )
            .map(move |cmd| cmd.map(move |msg| Message::VaultTab(index, msg)))
    }
}

/// The messages of the application.
#[derive(Clone, Debug)]
pub enum Message {
    /// Close the error dialog.
    ErrorDialogClose,
    /// Messages related to the password generator.
    PasswordGenerator(PasswordGeneratorMessage),
    /// Messages related to iced [`Event`](iced_native::Event)s.
    IcedEvent(iced_native::Event),
    /// Messages related to the tabs of the vaults.
    VaultTab(usize, VaultTabMessage),
    /// The tab identified by it's index was selected by the user.
    TabSelected(usize),
    /// Create a new tab.
    TabCreate,
    /// Close the tab identified by it's index.
    TabClose(usize),
    /// Open the settings tab.
    OpenSettings,
    /// Request focus in the given direction.
    Focus(iced_focus::Direction),
}

impl<P: Platform + 'static> Application for PWDuckGui<P> {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = pwduck_core::ApplicationSettings;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                modal_state: modal::State::new(ModalState::None),
                //tabs: vec![VaultTab::new(())],
                //selected_tab: 0,
                tabs: VaultTabVec::new(0, vec![VaultTab::new(())]),

                settings_state: button::State::new(),
                open_new_tab_state: button::State::new(),

                window_size: Viewport::default(),
                can_exit: false,
                phantom: PhantomData,

                application_settings: flags,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "PWDuck - Password Manager".into()
    }

    #[allow(clippy::print_stdout, clippy::use_debug)]
    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        let cmd = match message {
            Message::VaultTab(
                _,
                VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(
                    ModifyEntryMessage::PasswordGenerate,
                )),
                //) => Ok(self.password_generator_show(&message)),
            ) => Ok(
                PasswordGeneratorState::show(&message, &mut self.modal_state)
                    .map(Message::PasswordGenerator),
            ),

            Message::ErrorDialogClose => Ok(self.close_error_dialog()),

            Message::PasswordGenerator(PasswordGeneratorMessage::Cancel) => {
                //Ok(self.password_generator_cancel())
                Ok(PasswordGeneratorState::cancel(&mut self.modal_state)
                    .map(Message::PasswordGenerator))
            }

            Message::PasswordGenerator(PasswordGeneratorMessage::Submit) => {
                //self.password_generator_submit()
                let cmd = if let ModalState::Password(password_generator_state) =
                    self.modal_state.inner_mut()
                {
                    //password_generator_state.submit(self.selected_tab)
                    password_generator_state.submit(self.tabs.selected())
                } else {
                    Ok(Command::none())
                };

                if cmd.is_ok() {
                    self.modal_state = modal::State::default();
                }

                cmd
            }

            Message::PasswordGenerator(message) => {
                self.update_password_generator(message, clipboard)
            }

            Message::IcedEvent(event) => self.catch_iced_event(event),

            Message::VaultTab(index, message) => self.update_vault_tab(index, message, clipboard),

            Message::TabSelected(index) => {
                //self.selected_tab = index;
                self.tabs.select(index);
                Ok(Command::none())
            }

            Message::TabCreate => {
                self.tabs.push(VaultTab::new(()));
                //self.selected_tab = self.tabs.len() - 1;
                self.tabs.select(self.tabs.len() - 1);
                Ok(Command::none())
            }

            Message::TabClose(index) => {
                if self.tabs[index].contains_unsaved_changes() {
                    Err(PWDuckGuiError::VaultContainsUnsavedChanges)
                } else {
                    self.tabs.remove(index);
                    //self.selected_tab = if self.tabs.is_empty() {
                    //    0
                    //} else {
                    //    self.selected_tab.min(self.tabs.len() - 1)
                    //};
                    self.tabs.select(if self.tabs.is_empty() {
                        0
                    } else {
                        self.tabs.selected().min(self.tabs.len() - 1)
                    });

                    if self.tabs.is_empty() {
                        self.can_exit = true;
                    }
                    Ok(Command::none())
                }
            }

            Message::OpenSettings => {
                let mut settings_tab = VaultTab::new(());
                settings_tab.change_to_settings_state();
                self.tabs.push(settings_tab);
                //self.selected_tab = self.tabs.len() - 1;
                self.tabs.select(self.tabs.len() - 1);
                Ok(Command::none())
            }

            Message::Focus(direction) => {
                let _ = self.focus(direction);
                Ok(Command::none())
            }
        };

        match cmd {
            Ok(cmd) => cmd,
            Err(error) => {
                println!("{:?}", error);

                self.modal_state = modal::State::new(ModalState::Error(ErrorDialogState::new(
                    format!("{}", error),
                )));
                self.modal_state.show(true);

                Command::none()
            }
        }
    }

    #[cfg_attr(coverage, no_coverage)]
    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let theme: &dyn Theme = match self.application_settings.theme() {
            pwduck_core::theme::Theme::Light => &theme::Light,
            pwduck_core::theme::Theme::Dark => &theme::Dark,
        };

        if self.tabs.is_empty() {
            // Workaround to prevent rendering after the application should exit.
            return Column::new().into();
        }
        //let selected_tab = self.selected_tab;
        let selected_tab = self.tabs.selected();

        let top_row = Row::new()
            //.push(icon_button(&mut self.settings_state, Icon::Gear, "Settings", "Configure your preferences", true, None))
            .push(
                Tooltip::new(
                    Button::new(
                        &mut self.settings_state,
                        Text::new(Icon::Gear)
                            .font(ICON_FONT)
                            .size(TOP_ROW_FONT_SIZE),
                    )
                    .style(theme.button_primary())
                    .height(Length::Units(TOP_ROW_HEIGHT))
                    .padding(TOP_ROW_PADDING)
                    .on_press(Message::OpenSettings),
                    "Configure your preferences",
                    tooltip::Position::FollowCursor,
                )
                .style(theme.tooltip()),
            )
            //.push(icon_button(&mut self.open_new_tab_state, Icon::PlusSquare, "Open", "Open new tab", true, None)) // TODO
            .push(
                Tooltip::new(
                    Button::new(
                        &mut self.open_new_tab_state,
                        Text::new(Icon::PlusSquare)
                            .font(ICON_FONT)
                            .size(TOP_ROW_FONT_SIZE),
                    )
                    .style(theme.button_primary())
                    .height(Length::Units(TOP_ROW_HEIGHT))
                    .padding(TOP_ROW_PADDING)
                    .on_press(Message::TabCreate),
                    "Open new tab",
                    tooltip::Position::FollowCursor,
                )
                .style(theme.tooltip()),
            )
            .push(
                TabBar::width_tab_labels(
                    //self.selected_tab,
                    self.tabs.selected(),
                    self.tabs
                        .iter()
                        .map(|tab| TabLabel::Text(tab.title()))
                        .collect(),
                    Message::TabSelected,
                )
                .style(theme.tab_bar())
                .text_size(TOP_ROW_FONT_SIZE)
                .icon_size(TOP_ROW_FONT_SIZE)
                .padding(TOP_ROW_PADDING)
                .height(Length::Units(TOP_ROW_HEIGHT))
                .on_close(Message::TabClose),
            );

        let tab = self.tabs[selected_tab]
            .view::<P>(&self.application_settings, theme, &self.window_size)
            .map(move |msg| Message::VaultTab(selected_tab, msg));

        let content: Element<_> = Column::new().push(top_row).push(tab).into();

        let modal_style = match self.modal_state.inner() {
            ModalState::Password(_) => theme.modal(),
            _ => theme.modal_warning(),
        };
        Container::new(
            Modal::new(&mut self.modal_state, content, move |state| {
                state.view(selected_tab, theme)
            })
            .style(modal_style),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme.container())
        .into()
    }

    #[cfg_attr(coverage, no_coverage)]
    fn subscription(&self) -> Subscription<Self::Message> {
        //iced_native::subscription::events().map(Message::IcedEvent)
        iced_native::subscription::events_with(|event, _status| {
            if let iced_native::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code: iced_native::keyboard::KeyCode::Tab,
                modifiers,
            }) = event
            {
                Some(Message::Focus(if modifiers.shift {
                    iced_focus::Direction::Backwards
                } else {
                    iced_focus::Direction::Forwards
                }))
            } else {
                Some(Message::IcedEvent(event))
            }
        })
    }

    #[cfg_attr(coverage, no_coverage)]
    fn should_exit(&self) -> bool {
        self.can_exit
        //&& !self.tabs.iter().any(VaultTab::contains_unsaved_changes)
    }
}

/// Component trait to define components of this gui.
trait Component: iced_focus::Focus {
    /// Message produced by this [`Component`](Component).
    type Message: 'static;
    /// Parameters expected by the constructor of this [`Component`](Component).
    type ConstructorParam;

    /// Create a new [`Component`](Component).
    fn new(t: Self::ConstructorParam) -> Self;

    /// The title of this component.
    fn title(&self) -> String;

    /// Update this [`Component`](Component).
    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        application_settings: &mut pwduck_core::ApplicationSettings,
        modal_state: &mut modal::State<ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError>;

    /// Create the view of this [`Component`](Component).
    fn view<P: Platform + 'static>(
        &mut self,
        application_settings: &pwduck_core::ApplicationSettings,
        theme: &dyn Theme,
        viewport: &Viewport,
    ) -> iced::Element<'_, Self::Message>;
}

/// Platform related implementations.
#[async_trait]
pub trait Platform {
    /// True, if the native file dialog is available on this [`Platform`](Platform).
    fn is_nfd_available() -> bool;

    /// Open the choose folder dialog of the native file dialog on this [`Platform`](Platform).
    async fn nfd_choose_folder() -> Result<PathBuf, NfdError>;

    /// True, if the system supports to open an URL in the default browser.
    fn is_open_in_browser_available() -> bool;

    /// Open the given url in the default browser of the system.
    async fn open_in_browser(url: String) -> Result<(), PWDuckGuiError>;

    /// True, if the system supports autotyping.
    fn is_auto_type_available() -> bool;

    /// Autotype the given sequence.
    async fn auto_type(sequence: Sequence) -> Result<(), PWDuckGuiError>;
}

#[cfg(test)]
#[derive(Debug)]
pub struct TestPlatform;

#[cfg(test)]
#[async_trait]
impl Platform for TestPlatform {
    #[cfg_attr(coverage, no_coverage)]
    fn is_nfd_available() -> bool {
        true
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn nfd_choose_folder() -> Result<PathBuf, NfdError> {
        Ok(PathBuf::from("this/is/a/path"))
    }

    #[cfg_attr(coverage, no_coverage)]
    fn is_open_in_browser_available() -> bool {
        false
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn open_in_browser(_url: String) -> Result<(), PWDuckGuiError> {
        PWDuckGuiError::String("Not implemented".into()).into()
    }

    #[cfg_attr(coverage, no_coverage)]
    fn is_auto_type_available() -> bool {
        false
    }

    #[cfg_attr(coverage, no_coverage)]
    async fn auto_type(_sequence: Sequence) -> Result<(), PWDuckGuiError> {
        PWDuckGuiError::String("Not implemented".into()).into()
    }
}

/// The state of the modal.
#[derive(Debug)]
pub enum ModalState {
    /// The state of the error modal.
    Error(ErrorDialogState),
    /// The state of the password generator.
    Password(Box<PasswordGeneratorState>),
    /// The state of the group modifier modal.
    ModifyGroup(crate::vault::container::ModifyGroupModal),
    /// The state of the entry modifier modal.
    ModifyEntry(crate::vault::container::ModifyEntryModal),
    /// The modal is empty.
    None,
}

impl Default for ModalState {
    fn default() -> Self {
        Self::None
    }
}

impl ModalState {
    /// Create the view of the modal.
    #[cfg_attr(coverage, no_coverage)]
    fn view(&mut self, index: usize, theme: &dyn Theme) -> Element<'_, Message> {
        match self {
            ModalState::Error(error_modal_state) => error_modal_state.view(theme),
            ModalState::Password(password_generator_state) => password_generator_state
                .view(theme)
                .map(Message::PasswordGenerator),
            ModalState::ModifyGroup(modify_group_modal) => {
                modify_group_modal.view(theme).map(move |msg| {
                    Message::VaultTab(
                        index,
                        VaultTabMessage::Container(VaultContainerMessage::ModifyGroup(
                            ModifyGroupMessage::Modal(msg),
                        )),
                    )
                })
            }
            ModalState::ModifyEntry(modify_entry_modal) => {
                modify_entry_modal.view(theme).map(move |msg| {
                    Message::VaultTab(
                        index,
                        VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(
                            ModifyEntryMessage::Modal(msg),
                        )),
                    )
                })
            }
            ModalState::None => Text::new("This is a bug and should never be visible!").into(),
        }
    }
}
