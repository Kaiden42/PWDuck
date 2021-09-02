//! TODO
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![cfg_attr(not(test), forbid(unsafe_code))]
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
    clippy::module_name_repetitions,

    // TODO: remove
    clippy::missing_errors_doc,
)]

#[macro_use]
extern crate pest_derive;

mod auto_type;
pub use auto_type::{AutoTypeSequenceParser, Key, Part, Sequence};

mod cryptography;

mod error;
pub use error::PWDuckCoreError;

mod io;
pub use {io::load_application_settings, io::save_application_settings};

mod mem_protection;
pub use mem_protection::{try_to_prevent_core_dump, MemKey, SecString, SecVec};

mod model;
pub use model::{
    entry::{EntryBody, EntryHead},
    group::Group,
    master_key::MasterKey,
    settings::{theme, ApplicationSettings},
    uuid::{self, Uuid},
    vault::{ItemList, Vault},
};

mod passwords;
pub use passwords::{generate_password, password_entropy, Symbols};
pub use pw_entropy::PasswordInfo;

mod dto;
