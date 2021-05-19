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
    clippy::module_name_repetitions
)]

use std::{marker::PhantomData, path::PathBuf};

use async_trait::async_trait;
use iced::{executor, Application, Command, Settings, Subscription, Text};

pub mod error;
use error::{NfdError, PWDuckGuiError};

pub mod vault;
use iced_aw::{modal, Card, Modal};
use lazy_static::lazy_static;
use pwduck_core::MemKey;
use vault::{tab::VaultTab, tab::VaultTabMessage};

mod utils;

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
    tabs: Vec<VaultTab>,

    /// TODO
    windo_size: WindowSize,
    can_exit: bool,
    /// TODO
    phantom: PhantomData<P>,
}

#[derive(Debug, Default)]
struct ErrorDialogState {
    error: String,
}

#[derive(Debug, Default)]
struct WindowSize {
    width: u32,
    height: u32,
}

impl<P: Platform + 'static> PWDuckGui<P> {
    /// TODO
    pub fn start() -> Result<(), PWDuckGuiError> {
        //Self::run(Settings::default())?;
        Self::run(Settings {
            exit_on_close_request: false,
            ..Settings::default()
        })?;
        Ok(())
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum Message {
    /// TODO
    ErrorDialogClose,
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
            Message::ErrorDialogClose => {
                self.error_dialog_state.inner_mut().error.clear();
                self.error_dialog_state.show(false);
                Ok(Command::none())
            }

            Message::IcedEvent(event) => match event {
                iced_native::Event::Window(event) => match event {
                    iced_native::window::Event::Resized { width, height } => {
                        self.windo_size = WindowSize { width, height };
                        Ok(Command::none())
                    }
                    iced_native::window::Event::CloseRequested => {
                        if !self.tabs.iter().any(VaultTab::contains_unsaved_changes) {
                            // TODO: add nice error dialog
                            self.can_exit = true;
                        }
                        Ok(Command::none())
                    }
                    _ => Ok(Command::none()),
                },
                _ => Ok(Command::none()),
            },

            Message::VaultTab(msg) => self.tabs[0]
                .update::<P>(msg, clipboard)
                .map(|c| c.map(Message::VaultTab)),
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
        Modal::new(&mut self.error_dialog_state, tabs, |state| {
            Card::new(Text::new("Ooopsi whoopsi"), Text::new(state.error.clone()))
                .max_width(DEFAULT_MAX_WIDTH)
                .on_close(Message::ErrorDialogClose)
                .into()
        })
        .on_esc(Message::ErrorDialogClose)
        .backdrop(Message::ErrorDialogClose)
        .into()
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
