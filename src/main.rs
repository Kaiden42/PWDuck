#![windows_subsystem = "windows"]
//! TODO
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
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

use std::path::PathBuf;

use async_trait::async_trait;
use enigo::KeyboardControllable;
use pwduck_gui::{
    error::{NfdError, PWDuckGuiError},
    PWDuckGui, Platform, Sequence,
};
use rfd::AsyncFileDialog;

fn main() {
    PWDuckGui::<Desktop>::start().expect("Should not fail");
}

/// An empty placeholder struct to implement the [`Platform`](Platform) trait for.
#[derive(Default)]
struct Desktop;

#[async_trait]
impl Platform for Desktop {
    fn is_nfd_available() -> bool {
        true
    }

    async fn nfd_choose_folder() -> Result<PathBuf, pwduck_gui::error::NfdError> {
        let file = AsyncFileDialog::new()
            .set_directory(
                dirs::document_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| "".into())),
            )
            .pick_folder()
            .await
            .ok_or(NfdError::Null)?;

        Ok(file.path().into())
    }

    fn is_open_in_browser_available() -> bool {
        true
    }

    async fn open_in_browser(url: String) -> Result<(), pwduck_gui::error::PWDuckGuiError> {
        opener::open_browser(url)
            .map_err(|err| pwduck_gui::error::PWDuckGuiError::String(format!("{}", err)))?;
        Ok(())
    }

    fn is_auto_type_available() -> bool {
        true
    }

    async fn auto_type(sequence: Sequence) -> Result<(), pwduck_gui::error::PWDuckGuiError> {
        let mut enigo = enigo::Enigo::new();
        enigo.set_delay(25);

        if cfg!(target_os = "linux") {
            // Check if xdotools is available
            // TODO: maybe replace this by `get_program` in the future.
            // <https://doc.rust-lang.org/std/process/struct.Command.html#method.get_program>
            drop(
                std::process::Command::new("xdotool")
                    .output()
                    .map_err(|_err| PWDuckGuiError::XDOToolsMissing)?,
            );
        }

        async_std::task::sleep(std::time::Duration::from_millis(1000)).await;

        for part in sequence.iter() {
            match part {
                pwduck_gui::Part::Literal(literal) => enigo.key_sequence(literal),
                pwduck_gui::Part::Field(field) => enigo.key_sequence(field),
                pwduck_gui::Part::Key(key) => match key {
                    pwduck_gui::Key::Tab => enigo.key_click(enigo::Key::Tab),
                    pwduck_gui::Key::Return => enigo.key_click(enigo::Key::Return),
                },
            }
        }

        Ok(())
    }
}
