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
use iced::{executor, Application, Command, Element, Settings, Subscription, Text};

pub mod error;
use error::{NfdError, PWDuckGuiError};

pub mod vault;
use iced_aw::{modal, Card, Modal};
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

/// TODO
const DEFAULT_MAX_WIDTH: u32 = 600;
/// TODO
const DEFAULT_COLUMN_PADDING: u16 = 16;
/// TODO
const DEFAULT_COLUMN_SPACING: u16 = 5;
/// TODO
const DEFAULT_ROW_SPACING: u16 = 5;
/// TODO
const DEFAULT_TEXT_INPUT_PADDING: u16 = 5;
/// TODO
const DEFAULT_SPACE_HEIGHT: u16 = 5;
/// TODO
const DEFAULT_HEADER_SIZE: u16 = 25;

lazy_static! {
    //static ref MEM_KEY: Mutex<pwduck_core::MemKey> = Mutex::new(MemKey::new());
    static ref MEM_KEY: std::sync::Mutex<pwduck_core::MemKey> = std::sync::Mutex::new(MemKey::new());

    //static ref MEM_KEY: Arc<RwLock<pwduck_core::MemKey>> = Arc::new(RwLock::new(MemKey::new()));
}

/// TODO
#[derive(Debug)]
pub struct PWDuckGui<P: Platform + 'static> {
    /// TODO
    error_dialog_state: modal::State<ErrorDialogState>,
    /// TODO
    password_generator_state: modal::State<PasswordGeneratorState>,
    /// TODO
    tabs: Vec<VaultTab>,

    /// TODO
    windo_size: WindowSize,
    /// TODO
    can_exit: bool,

    /// TODO
    phantom: PhantomData<P>,
}

/// TODO
#[derive(Debug, Default)]
struct ErrorDialogState {
    /// TODO
    error: String,
}

/// TODO
#[derive(Debug, Default)]
struct WindowSize {
    /// TODO
    width: u32,
    /// TODO
    height: u32,
}

impl<P: Platform + 'static> PWDuckGui<P> {
    /// TODO
    pub fn start() -> Result<(), PWDuckGuiError> {
        pwduck_core::try_to_prevent_core_dump()?;

        //Self::run(Settings::default())?;
        Self::run(Settings {
            exit_on_close_request: false,
            ..Settings::default()
        })?;
        Ok(())
    }

    /// TODO
    fn password_generator_show(&mut self, message: &Message) -> iced::Command<Message> {
        self.password_generator_state
            .inner_mut()
            .generate_and_update_password();
        self.password_generator_state.show(true);

        self.password_generator_state
            .inner_mut()
            .set_target(match message {
                Message::VaultTab(VaultTabMessage::Container(
                    VaultContainerMessage::ModifyEntry(_),
                )) => Target::EntryModifier,
                Message::VaultTab(VaultTabMessage::Creator(_)) => todo!(),
                _ => Target::None,
            });

        // TODO: clean up
        Command::perform(
            estimate_password_strength(self.password_generator_state.inner().password().clone()),
            PasswordGeneratorMessage::PasswordScore,
        )
        .map(Message::PasswordGenerator)
    }

    /// TODO
    fn password_generator_cancel(&mut self) -> iced::Command<Message> {
        self.password_generator_state.show(false);
        Command::none()
    }

    /// TODO
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
            async { Message::VaultTab(message) },
            |m| m,
        ))
    }

    /// TODO
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

    /// TODO
    fn close_error_dialog(&mut self) -> iced::Command<Message> {
        self.error_dialog_state.inner_mut().error.clear();
        self.error_dialog_state.show(false);
        Command::none()
    }

    /// TODO
    fn catch_iced_event<Message>(&mut self, event: iced_native::Event) -> iced::Command<Message> {
        match event {
            iced_native::Event::Window(event) => match event {
                iced_native::window::Event::Resized { width, height } => {
                    self.windo_size = WindowSize { width, height };
                    Command::none()
                }
                iced_native::window::Event::CloseRequested => {
                    if !self.tabs.iter().any(VaultTab::contains_unsaved_changes) {
                        // TODO: add nice error dialog
                        self.can_exit = true;
                    }
                    Command::none()
                }
                _ => Command::none(),
            },
            _ => Command::none(),
        }
    }

    /// TODO
    fn update_vault_tab(
        &mut self,
        message: VaultTabMessage,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Message>, PWDuckGuiError> {
        self.tabs[0]
            .update::<P>(message, clipboard)
            .map(|c| c.map(Message::VaultTab))
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum Message {
    /// TODO
    ErrorDialogClose,
    /// TODO
    PasswordGenerator(PasswordGeneratorMessage),
    /// TODO
    IcedEvent(iced_native::Event),
    /// TODO
    VaultTab(VaultTabMessage),
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
                windo_size: WindowSize::default(),
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
            Message::VaultTab(VaultTabMessage::Container(VaultContainerMessage::ModifyEntry(
                ModifyEntryMessage::PasswordGenerate,
            ))) => Ok(self.password_generator_show(&message)),

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

            Message::IcedEvent(event) => Ok(self.catch_iced_event(event)),

            Message::VaultTab(message) => self.update_vault_tab(message, clipboard),
        };

        match cmd {
            Ok(cmd) => cmd,
            Err(error) => {
                self.error_dialog_state.inner_mut().error = format!("{:?}", error);
                self.error_dialog_state.show(true);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let tabs = self.tabs[0].view::<P>().map(Message::VaultTab);

        let body = password_modal::<P>(&mut self.password_generator_state, tabs);

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

/// TODO
fn password_modal<'a, P: Platform + 'static>(
    state: &'a mut modal::State<pw_modal::PasswordGeneratorState>,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    Modal::new(state, body, |state| {
        state.view().map(Message::PasswordGenerator)
    })
    .into()
}

/// TODO
fn error_modal<'a, P: Platform + 'static>(
    state: &'a mut modal::State<ErrorDialogState>,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    Modal::new(state, body, |state| {
        Card::new(Text::new("Ooopsi whoopsi"), Text::new(state.error.clone()))
            .max_width(DEFAULT_MAX_WIDTH)
            .on_close(Message::ErrorDialogClose)
            .into()
    })
    .on_esc(Message::ErrorDialogClose)
    .backdrop(Message::ErrorDialogClose)
    .into()
}

/// TODO
trait Component {
    /// TODO
    type Message: 'static;
    /// TODO
    type ConstructorParam;

    /// TODO
    fn new(t: Self::ConstructorParam) -> Self;

    /// TODO
    fn update<P: Platform + 'static>(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Result<iced::Command<Self::Message>, PWDuckGuiError>;

    /// TODO
    fn view<P: Platform + 'static>(&mut self) -> iced::Element<'_, Self::Message>;
}

/// TODO
#[async_trait]
pub trait Platform {
    /// TODO
    //fn new() -> Self;

    /// TODO
    fn is_nfd_available() -> bool;

    /// TODO
    async fn nfd_choose_folder() -> Result<PathBuf, NfdError>;
}
