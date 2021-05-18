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
use iced::{executor, Application, Command, Settings};

pub mod error;
use error::{NfdError, PWDuckGuiError};

pub mod vault;
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
    tabs: Vec<VaultTab>,
    /// TODO
    phantom: PhantomData<P>,
}

impl<P: Platform + 'static> PWDuckGui<P> {
    /// TODO
    pub fn start() -> Result<(), PWDuckGuiError> {
        Self::run(Settings::default())?;
        Ok(())
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum Message {
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
                tabs: vec![VaultTab::new(())],
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
        match message {
            Message::VaultTab(msg) => self.tabs[0]
                .update::<P>(msg, clipboard)
                .map(Message::VaultTab),
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        self.tabs[0].view::<P>().map(Message::VaultTab)
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
    ) -> iced::Command<Self::Message>;

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