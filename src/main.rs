//! A password manager written in Rust using Iced as a graphical user interface.
//!
//! This module is the starting point of the application. It uses the [gui](pwduck_gui) module
//! to open up a graphical user interface.
#![windows_subsystem = "windows"]
#![cfg_attr(coverage, feature(no_coverage))]
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

use desktop::Desktop;
use pwduck_gui::PWDuckGui;

fn main() {
    PWDuckGui::<Desktop>::start().expect("Should not fail");
}

/// [`Platform`](pwduck_gui::Platform) implementation for the [`Desktop`](Desktop) platform.
mod desktop {

    use std::path::PathBuf;

    use async_trait::async_trait;
    use enigo::KeyboardControllable;
    use rfd::AsyncFileDialog;

    use pwduck_gui::{
        error::{NfdError, PWDuckGuiError},
        Platform, Sequence,
    };

    /// An empty placeholder struct to implement the [`Platform`](Platform) trait for.
    #[derive(Default)]
    pub struct Desktop;

    #[async_trait]
    impl Platform for Desktop {
        fn is_nfd_available() -> bool {
            true
        }

        async fn nfd_choose_folder() -> Result<PathBuf, pwduck_gui::error::NfdError> {
            let folder = AsyncFileDialog::new()
                .set_directory(
                    dirs::document_dir()
                        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| "".into())),
                )
                .pick_folder()
                .await
                .ok_or(NfdError::Null)?;

            Ok(folder.path().into())
        }

        async fn nfd_choose_key_file(
            file_name: Option<String>,
        ) -> Result<PathBuf, pwduck_gui::error::NfdError> {
            let key_file = AsyncFileDialog::new()
                .set_directory(
                    dirs::document_dir()
                        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| "".into())),
                )
                .add_filter("PWDuck key file", &["pwdk"]);

            let key_file = if let Some(file_name) = file_name {
                key_file.set_file_name(&file_name).save_file().await
            } else {
                key_file.pick_file().await
            }
            .ok_or(NfdError::Null)?;

            Ok(key_file.path().into())
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
            async_std::task::sleep(std::time::Duration::from_millis(1000)).await;

            let mut enigo = enigo::Enigo::new();
            #[cfg(target_os = "linux")]
            {
                enigo.set_delay(25);

                // Check if xdotools is available
                drop(which::which("xdotool")
                    .map_err(|err| PWDuckGuiError::String(format!("xdotool could not be found. Maybe it is not installed on your system? ({})", err)))?);
            }

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
}
