//! The graphical user interface (GUI) of the password manager.
//!
//! It uses the [core](pwduck_core) module internally to manage the passwords of the user.
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
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
    clippy::unwrap_used,
    clippy::use_debug,
)]
#![allow(
    clippy::suboptimal_flops,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::module_name_repetitions
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

#[cfg(test)]
use mocktopus::macros::*;

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
    /// The memory key
    static ref MEM_KEY: std::sync::Mutex<pwduck_core::MemKey> = std::sync::Mutex::new(MemKey::new());
}

/// The state of the GUI.
#[derive(Debug, Focus)]
pub struct PWDuckGui<P: Platform + 'static> {
    /// The state of the modal
    modal_state: modal::State<ModalState>,
    /// The tabs of open vaults.
    #[focus(enable)]
    tabs: VaultTabVec,

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

#[cfg_attr(test, mockable)]
impl<P: Platform + 'static> PWDuckGui<P> {
    /// Start the gui application.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - The core dump prevention fails.
    /// - The application can't be started.
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

    /// Select the tab identified by the tab index.
    fn select_tab(&mut self, index: usize) -> Command<Message> {
        self.tabs.select(index);
        Command::none()
    }

    /// Create a new tab.
    fn create_tab(&mut self) -> Command<Message> {
        self.tabs.push(VaultTab::new(()));
        self.tabs.select(self.tabs.len() - 1);
        Command::none()
    }

    /// Close the tab identified by the tab index.
    fn close_tab(&mut self, index: usize) -> Result<Command<Message>, PWDuckGuiError> {
        if self.tabs[index].contains_unsaved_changes() {
            Err(PWDuckGuiError::VaultContainsUnsavedChanges)
        } else {
            drop(self.tabs.remove(index));

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

    /// Open the settings tab.
    fn open_settings(&mut self) -> Command<Message> {
        let mut settings_tab = VaultTab::new(());
        let cmd = settings_tab.change_to_settings_state();
        self.tabs.push(settings_tab);
        self.tabs.select(self.tabs.len() - 1);
        let selected = self.tabs.selected();
        cmd.map(move |msg| Message::VaultTab(selected, msg))
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

    #[cfg_attr(coverage, no_coverage)]
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
            ) => Ok(
                PasswordGeneratorState::show(&message, &mut self.modal_state)
                    .map(Message::PasswordGenerator),
            ),

            Message::ErrorDialogClose => Ok(self.close_error_dialog()),

            Message::PasswordGenerator(PasswordGeneratorMessage::Cancel) => {
                Ok(PasswordGeneratorState::cancel(&mut self.modal_state)
                    .map(Message::PasswordGenerator))
            }

            Message::PasswordGenerator(PasswordGeneratorMessage::Submit) => {
                let cmd = if let ModalState::Password(password_generator_state) =
                    self.modal_state.inner_mut()
                {
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

            Message::TabSelected(index) => Ok(self.select_tab(index)),

            Message::TabCreate => Ok(self.create_tab()),

            Message::TabClose(index) => self.close_tab(index),

            Message::OpenSettings => Ok(self.open_settings()),

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
        let selected_tab = self.tabs.selected();

        let top_row = Row::new()
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

    /// Open the choose file dialog of the native file dialog on this [`Platform`](Platform).
    /// If file name is some a file save dialoge is used, else a file open dialog.
    async fn nfd_choose_key_file(file_name: Option<String>) -> Result<PathBuf, NfdError>;

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
    async fn nfd_choose_key_file(_file_name: Option<String>) -> Result<PathBuf, NfdError> {
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

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
    };

    use crate::{
        error::PWDuckGuiError,
        pw_modal::PasswordGeneratorState,
        vault::tab::{VaultTab, VaultTabMessage},
        Component, ErrorDialogState,
    };
    use iced::{Application, Command};
    use mocktopus::mocking::*;

    use super::{Message, ModalState, PWDuckGui, TestPlatform};
    use pwduck_core::ApplicationSettings;

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    fn default_pwduck_gui() -> PWDuckGui<TestPlatform> {
        let application_settings = ApplicationSettings::default();
        PWDuckGui::new(application_settings).0
    }

    #[test]
    fn update_password_generator() {
        use crate::PasswordGeneratorMessage;
        let mut gui = default_pwduck_gui();

        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::update.type_id(), 0);

            PasswordGeneratorState::update.mock_raw(|_self, _message, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::update.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            if let ModalState::None = gui.modal_state.inner() {
            } else {
                panic!("Modal state is not None");
            }

            // Update a non existent generator should be ignored
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update.type_id()],
                0
            );
            let _ = gui.update_password_generator(PasswordGeneratorMessage::Cancel, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update.type_id()],
                0
            );

            // Update an existent generator should not be ignored
            gui.modal_state = iced_aw::modal::State::new(ModalState::Password(Box::new(
                PasswordGeneratorState::new(),
            )));
            let _ = gui.update_password_generator(PasswordGeneratorMessage::Cancel, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::update.type_id()],
                1
            );
        })
    }

    #[test]
    fn close_error_dialog() {
        let mut gui = default_pwduck_gui();

        gui.modal_state =
            iced_aw::modal::State::new(ModalState::Error(ErrorDialogState::new("Error".into())));

        if let ModalState::None = gui.modal_state.inner() {
            panic!("Modal state should not be None");
        }

        gui.close_error_dialog();

        if let ModalState::None = gui.modal_state.inner() {
        } else {
            panic!("Modal state should be None");
        }
    }

    #[test]
    fn select_tab() {
        let mut gui = default_pwduck_gui();

        for _ in 0..10 {
            let _ = gui.create_tab();
        }

        assert_eq!(gui.tabs.selected(), 10);

        gui.select_tab(3);

        assert_eq!(gui.tabs.selected(), 3);
    }

    #[test]
    fn create_tab() {
        let mut gui = default_pwduck_gui();

        assert_eq!(gui.tabs.selected(), 0);

        for i in 1..10 {
            gui.create_tab();
            assert_eq!(gui.tabs.selected(), i);

            // TODO: check state of tab
            //let tab = gui.tabs[i];
            //if let crate::vault::tab::VaultTabState::Empty(..) = tab.state {
            //} else {
            //    panic!("Newly created tab should be empty");
            //}
        }
    }

    #[test]
    fn close_tab() {
        let mut gui = default_pwduck_gui();

        // Tab without unsaved changes should be closable.
        VaultTab::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));

        for _ in 0..10 {
            let _ = gui.create_tab();
        }

        assert_eq!(gui.tabs.selected(), 10);
        assert_eq!(gui.tabs.len(), 11);

        let _ = gui.select_tab(5);
        let _ = gui.close_tab(5).expect("Should not fail");

        assert_eq!(gui.tabs.selected(), 5);
        assert_eq!(gui.tabs.len(), 10);

        let _ = gui.select_tab(9);
        let _ = gui.close_tab(9).expect("Should not fail");

        assert_eq!(gui.tabs.selected(), 8);
        assert_eq!(gui.tabs.len(), 9);

        let _ = gui.select_tab(3);
        let _ = gui.close_tab(6).expect("Should not fail");

        assert_eq!(gui.tabs.selected(), 3);
        assert_eq!(gui.tabs.len(), 8);

        // Tab with unsaved changes should not be closable.
        VaultTab::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(true));

        let _ = gui.close_tab(0).expect_err("Should fail");

        VaultTab::contains_unsaved_changes.mock_safe(|_self| MockResult::Return(false));

        // Application can exit when no tab remains
        assert!(!gui.can_exit);
        for _ in 0..7 {
            let _ = gui.close_tab(0);
            assert!(!gui.can_exit);
        }
        assert_eq!(gui.tabs.len(), 1);
        let _ = gui.close_tab(0);
        assert!(gui.can_exit);
    }

    #[test]
    fn update_vault_tab() {
        let mut gui = default_pwduck_gui();

        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(VaultTab::update::<TestPlatform>.type_id(), 0);

            VaultTab::update::<TestPlatform>.mock_raw(|_self, _m, _a, _s, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&VaultTab::update::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            assert_eq!(
                call_map.borrow()[&VaultTab::update::<TestPlatform>.type_id()],
                0
            );
            let _ = gui.update_vault_tab(
                0,
                VaultTabMessage::Loader(crate::vault::loader::VaultLoaderMessage::Submit),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&VaultTab::update::<TestPlatform>.type_id()],
                1
            );
        });
    }

    #[test]
    fn open_settings() {
        let mut gui = default_pwduck_gui();

        assert_eq!(gui.tabs.len(), 1);

        let _ = gui.open_settings();

        assert_eq!(gui.tabs.len(), 2);

        // TODO: check state of tab.
    }

    #[test]
    fn new() {
        let application_settings = ApplicationSettings::default();
        let gui: PWDuckGui<TestPlatform> = PWDuckGui::new(application_settings).0;

        if let crate::ModalState::None = gui.modal_state.inner() {
        } else {
            panic!("Modal state should be None");
        }

        assert_eq!(gui.tabs.len(), 1);

        assert_eq!(gui.window_size.width, 0);
        assert_eq!(gui.window_size.height, 0);

        assert!(!gui.can_exit);
    }

    #[test]
    fn update() {
        use crate::{
            pw_modal::{PasswordGeneratorMessage, PasswordGeneratorState},
            vault::{
                container::{ModifyEntryMessage, VaultContainerMessage},
                loader::VaultLoaderMessage,
            },
        };

        let mut gui = default_pwduck_gui();

        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::show.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PWDuckGui::<TestPlatform>::close_error_dialog.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::cancel.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PasswordGeneratorState::submit.type_id(), 0);
            call_map.borrow_mut().insert(
                PWDuckGui::<TestPlatform>::update_password_generator.type_id(),
                0,
            );
            call_map.borrow_mut().insert(
                PWDuckGui::<TestPlatform>::catch_iced_event::<Message>.type_id(),
                0,
            );
            call_map
                .borrow_mut()
                .insert(PWDuckGui::<TestPlatform>::update_vault_tab.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PWDuckGui::<TestPlatform>::select_tab.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PWDuckGui::<TestPlatform>::create_tab.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PWDuckGui::<TestPlatform>::close_tab.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(PWDuckGui::<TestPlatform>::open_settings.type_id(), 0);

            PasswordGeneratorState::show.mock_raw(|_message, _modal_state| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::show.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PWDuckGui::<TestPlatform>::close_error_dialog.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::close_error_dialog.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::cancel.mock_raw(|_modal_state| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::cancel.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PasswordGeneratorState::submit.mock_raw(|_self, _index| {
                call_map
                    .borrow_mut()
                    .get_mut(&PasswordGeneratorState::submit.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            PWDuckGui::<TestPlatform>::update_password_generator.mock_raw(|_self, _m, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::update_password_generator.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            PWDuckGui::<TestPlatform>::catch_iced_event::<Message>.mock_raw(|_self, _event| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::catch_iced_event::<Message>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            PWDuckGui::<TestPlatform>::update_vault_tab.mock_raw(|_self, _i, _m, _c| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::update_vault_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            PWDuckGui::<TestPlatform>::select_tab.mock_raw(|_self, _index| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::select_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PWDuckGui::<TestPlatform>::create_tab.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::create_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            PWDuckGui::<TestPlatform>::close_tab.mock_raw(|_self, _index| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::close_tab.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });
            PWDuckGui::<TestPlatform>::open_settings.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&PWDuckGui::<TestPlatform>::open_settings.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Show password generator
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::show.type_id()],
                0
            );
            let _ = gui.update(
                Message::VaultTab(
                    0,
                    VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(
                        ModifyEntryMessage::PasswordGenerate,
                    )),
                ),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::show.type_id()],
                1
            );

            // Close error dialog
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::close_error_dialog.type_id()],
                0
            );
            let _ = gui.update(Message::ErrorDialogClose, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::close_error_dialog.type_id()],
                1
            );

            // Cancel password generator
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::cancel.type_id()],
                0
            );
            let _ = gui.update(
                Message::PasswordGenerator(PasswordGeneratorMessage::Cancel),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::cancel.type_id()],
                1
            );

            // Submit password generator
            if let ModalState::None = gui.modal_state.inner() {
            } else {
                panic!("Modal state should be None");
            }
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::submit.type_id()],
                0
            );
            let _ = gui.update(
                Message::PasswordGenerator(PasswordGeneratorMessage::Submit),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::submit.type_id()],
                0
            );

            gui.modal_state = iced_aw::modal::State::new(ModalState::Password(Box::new(
                PasswordGeneratorState::new(),
            )));
            if let ModalState::Password(..) = gui.modal_state.inner() {
            } else {
                panic!("Modal state should be a password generator");
            }

            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::submit.type_id()],
                0
            );
            let _ = gui.update(
                Message::PasswordGenerator(PasswordGeneratorMessage::Submit),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PasswordGeneratorState::submit.type_id()],
                1
            );
            if let ModalState::None = gui.modal_state.inner() {
            } else {
                panic!("Modal state should be None");
            }

            // Update password generator
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::update_password_generator.type_id()],
                0
            );
            let _ = gui.update(
                Message::PasswordGenerator(PasswordGeneratorMessage::PasswordCopy),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::update_password_generator.type_id()],
                1
            );

            // TODO: Catch iced event

            // Update vault tab
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::update_vault_tab.type_id()],
                0
            );
            let _ = gui.update(
                Message::VaultTab(0, VaultTabMessage::Loader(VaultLoaderMessage::Submit)),
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::update_vault_tab.type_id()],
                1
            );

            // Select tab
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::select_tab.type_id()],
                0
            );
            let _ = gui.update(Message::TabSelected(2), &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::select_tab.type_id()],
                1
            );

            // Create tab
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::create_tab.type_id()],
                0
            );
            let _ = gui.update(Message::TabCreate, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::create_tab.type_id()],
                1
            );

            // Close tab
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::close_tab.type_id()],
                0
            );
            let _ = gui.update(Message::TabClose(1), &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::close_tab.type_id()],
                1
            );

            // Open settings
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::open_settings.type_id()],
                0
            );
            let _ = gui.update(Message::OpenSettings, &mut clipboard);
            assert_eq!(
                call_map.borrow()[&PWDuckGui::<TestPlatform>::open_settings.type_id()],
                1
            );

            // Focus
            let _ = gui.update(
                Message::Focus(iced_focus::Direction::Forwards),
                &mut clipboard,
            );

            //assert!(call_map.borrow().values().all(|v| *v == 1))

            // Check correct error handling.
            PWDuckGui::<TestPlatform>::update_vault_tab.mock_raw(|_self, _i, _m, _c| {
                MockResult::Return(Err(PWDuckGuiError::String("Error".into())))
            });
            if let ModalState::None = gui.modal_state.inner() {
            } else {
                panic!("Modal state should be None");
            }

            let _ = gui.update(
                Message::VaultTab(0, VaultTabMessage::Loader(VaultLoaderMessage::Submit)),
                &mut clipboard,
            );

            if let ModalState::Error(..) = gui.modal_state.inner() {
            } else {
                panic!("Modal state should be an error message");
            }
        });
    }
}
