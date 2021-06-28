//! This module contains everything related to errors occurring in the gui.

use std::sync::PoisonError;

use pwduck_core::PWDuckCoreError;

/// An error thrown in the gui.
#[derive(Debug)]
pub enum PWDuckGuiError {
    /// An error thrown by Iced.
    Iced(iced::Error),
    /// Locking a mutex failed.
    Mutex(String),
    /// An expected `Some(_)` was not present.
    Option,
    /// An error bubbled up from the core.
    PWDuckCoreError(PWDuckCoreError),
    /// An unreachable path was reached.
    Unreachable(String),
}

impl From<iced::Error> for PWDuckGuiError {
    fn from(error: iced::Error) -> Self {
        Self::Iced(error)
    }
}

impl<T> From<PoisonError<T>> for PWDuckGuiError {
    fn from(error: PoisonError<T>) -> Self {
        Self::Mutex(format!("{:?}", error))
    }
}

impl From<PWDuckCoreError> for PWDuckGuiError {
    fn from(error: PWDuckCoreError) -> Self {
        Self::PWDuckCoreError(error)
    }
}

/// An error thrown by the natve file dialog.
#[derive(Clone, Debug)]
pub enum NfdError {
    /// The native file dialog was not available.
    Null,
}

impl<T> From<PWDuckGuiError> for Result<T, PWDuckGuiError> {
    fn from(error: PWDuckGuiError) -> Self {
        Err(error)
    }
}
