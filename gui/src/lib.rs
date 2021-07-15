//! TODO
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
//#![deny(unused_results)]
#![forbid(unsafe_code)]
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
    button, executor, tooltip, Application, Button, Column, Command, Element, Length, Row,
    Settings, Subscription, Text, Tooltip,
};

pub mod error;
use error::{NfdError, PWDuckGuiError};

pub mod vault;
use iced_aw::{modal, Card, Modal, TabBar, TabLabel};
use icons::{Icon, ICON_FONT};
use lazy_static::lazy_static;
use pwduck_core::MemKey;
use vault::{
    container::ModifyEntryMessage,
    tab::VaultTabMessage,
    tab::{VaultContainerMessage, VaultTab},
};

mod pw_modal;
mod utils;

use pw_modal::{PasswordGeneratorMessage, PasswordGeneratorState, Target};

use crate::{utils::estimate_password_strength, vault::creator::VaultCreatorMessage};

mod password_score;

mod icons;

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
#[derive(Debug)]
pub struct PWDuckGui<P: Platform + 'static> {
    /// The state of the error dialog.
    error_dialog_state: modal::State<ErrorDialogState>,
    /// The state of the password generator.
    password_generator_state: modal::State<PasswordGeneratorState>,
    /// The tabs of open vaults.
    tabs: Vec<VaultTab>,
    /// The index of the currently selected tab.
    selected_tab: usize,

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
}

/// The state of the error dialog.
#[derive(Debug, Default)]
struct ErrorDialogState {
    /// The text of the error.
    error: String,
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
    pub fn start() -> Result<(), PWDuckGuiError> {
        pwduck_core::try_to_prevent_core_dump()?;

        //Self::run(Settings::default())?;
        Self::run(Settings {
            exit_on_close_request: false,
            ..Settings::default()
        })?;
        Ok(())
    }

    /// Show the password generator.
    fn password_generator_show(&mut self, message: &Message) -> iced::Command<Message> {
        self.password_generator_state
            .inner_mut()
            .generate_and_update_password();
        self.password_generator_state.show(true);

        self.password_generator_state
            .inner_mut()
            .set_target(match message {
                Message::VaultTab(
                    _,
                    VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(_)),
                ) => Target::EntryModifier,
                Message::VaultTab(_, VaultTabMessage::Creator(_)) => todo!(),
                _ => Target::None,
            });

        // TODO: clean up
        Command::perform(
            estimate_password_strength(self.password_generator_state.inner().password().clone()),
            PasswordGeneratorMessage::PasswordScore,
        )
        .map(Message::PasswordGenerator)
    }

    /// Hide the password generator.
    fn password_generator_cancel(&mut self) -> iced::Command<Message> {
        self.password_generator_state.show(false);
        Command::none()
    }

    /// Process the submission of the password generator.
    fn password_generator_submit(&mut self) -> Result<iced::Command<Message>, PWDuckGuiError> {
        self.password_generator_state.show(false);
        // TODO: clean up
        let password = self.password_generator_state.inner().password().clone();
        let message = match self.password_generator_state.inner().target() {
            Target::Creator => {
                VaultTabMessage::Creator(VaultCreatorMessage::PasswordInput(password.into()))
            }
            Target::EntryModifier => {
                VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(
                    ModifyEntryMessage::PasswordInput(password.into()),
                ))
            }
            Target::None => return PWDuckGuiError::Unreachable("Message".into()).into(),
        };
        Ok(Command::perform(
            {
                let selected_tab = self.selected_tab;
                async move { Message::VaultTab(selected_tab, message) }
            },
            |m| m,
        ))
    }

    /// Update the state of the password generator.
    fn update_password_generator(
        &mut self,
        message: PasswordGeneratorMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Message>, PWDuckGuiError> {
        self.password_generator_state
            .inner_mut()
            .update(message, clipboard)
            .map(|cmd| cmd.map(Message::PasswordGenerator))
    }

    /// Hide the error dialog.
    fn close_error_dialog(&mut self) -> iced::Command<Message> {
        self.error_dialog_state.inner_mut().error.clear();
        self.error_dialog_state.show(false);
        Command::none()
    }

    /// Catch and handle an [`Event`](iced_native::Event) thrown by iced.
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
            .update::<P>(message, clipboard)
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
}

impl<P: Platform + 'static> Application for PWDuckGui<P> {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                error_dialog_state: modal::State::default(),
                password_generator_state: modal::State::new(PasswordGeneratorState::new()),
                tabs: vec![VaultTab::new(())],
                selected_tab: 0,

                settings_state: button::State::new(),
                open_new_tab_state: button::State::new(),

                window_size: Viewport::default(),
                can_exit: false,
                phantom: PhantomData,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "PWDuck - Password Manager".into()
    }

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
            ) => Ok(self.password_generator_show(&message)),

            Message::ErrorDialogClose => Ok(self.close_error_dialog()),

            Message::PasswordGenerator(PasswordGeneratorMessage::Cancel) => {
                Ok(self.password_generator_cancel())
            }

            Message::PasswordGenerator(PasswordGeneratorMessage::Submit) => {
                self.password_generator_submit()
            }

            Message::PasswordGenerator(message) => {
                self.update_password_generator(message, clipboard)
            }

            Message::IcedEvent(event) => self.catch_iced_event(event),

            Message::VaultTab(index, message) => self.update_vault_tab(index, message, clipboard),

            Message::TabSelected(index) => {
                self.selected_tab = index;
                Ok(Command::none())
            }

            Message::TabCreate => {
                self.tabs.push(VaultTab::new(()));
                self.selected_tab = self.tabs.len() - 1;
                Ok(Command::none())
            }

            Message::TabClose(index) => {
                if self.tabs[index].contains_unsaved_changes() {
                    Err(PWDuckGuiError::VaultContainsUnsavedChanges)
                } else {
                    self.tabs.remove(index);
                    self.selected_tab = if self.tabs.is_empty() {
                        0
                    } else {
                        self.selected_tab.min(self.tabs.len() - 1)
                    };

                    if self.tabs.is_empty() {
                        self.can_exit = true;
                    }
                    Ok(Command::none())
                }
            }
        };

        match cmd {
            Ok(cmd) => cmd,
            Err(error) => {
                println!("{:?}", error);
                self.error_dialog_state.inner_mut().error = format!("{}", error);
                self.error_dialog_state.show(true);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        if self.tabs.is_empty() {
            // Workaround to prevent rendering after the application should exit.
            return Column::new().into();
        }
        let selected_tab = self.selected_tab;

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
                    .height(Length::Units(TOP_ROW_HEIGHT))
                    .padding(TOP_ROW_PADDING),
                    "Configure your preferences",
                    tooltip::Position::FollowCursor,
                )
                .style(crate::utils::TooltipStyle),
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
                    .height(Length::Units(TOP_ROW_HEIGHT))
                    .padding(TOP_ROW_PADDING)
                    .on_press(Message::TabCreate),
                    "Open new tab",
                    tooltip::Position::FollowCursor,
                )
                .style(crate::utils::TooltipStyle),
            )
            .push(
                TabBar::width_tab_labels(
                    self.selected_tab,
                    self.tabs
                        .iter()
                        .map(|tab| TabLabel::Text(tab.title()))
                        .collect(),
                    Message::TabSelected,
                )
                .text_size(TOP_ROW_FONT_SIZE)
                .icon_size(TOP_ROW_FONT_SIZE)
                .padding(TOP_ROW_PADDING)
                .height(Length::Units(TOP_ROW_HEIGHT))
                .on_close(Message::TabClose),
            );

        let tab = self.tabs[selected_tab]
            .view::<P>(&self.window_size)
            .map(move |msg| Message::VaultTab(selected_tab, msg));

        let content: Element<_> = Column::new().push(top_row).push(tab).into();

        let body = password_modal::<P>(&mut self.password_generator_state, content);

        error_modal::<P>(&mut self.error_dialog_state, body)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::IcedEvent)
    }

    fn should_exit(&self) -> bool {
        self.can_exit
        //&& !self.tabs.iter().any(VaultTab::contains_unsaved_changes)
    }
}

/// Create the view of the password generator.
fn password_modal<'a, P: Platform + 'static>(
    state: &'a mut modal::State<pw_modal::PasswordGeneratorState>,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    Modal::new(state, body, |state| {
        state.view().map(Message::PasswordGenerator)
    })
    .into()
}

/// Create the view of the error dialog.
fn error_modal<'a, P: Platform + 'static>(
    state: &'a mut modal::State<ErrorDialogState>,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    Modal::new(state, body, |state| {
        Card::new(
            Text::new("An error occurred"),
            Text::new(state.error.clone()),
        )
        .max_width(DEFAULT_MAX_WIDTH)
        .on_close(Message::ErrorDialogClose)
        .into()
    })
    .on_esc(Message::ErrorDialogClose)
    .backdrop(Message::ErrorDialogClose)
    .into()
}

/// Component trait to define components of this gui.
trait Component {
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
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError>;

    /// Create the view of this [`Component`](Component).
    fn view<P: Platform + 'static>(
        &mut self,
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
}
