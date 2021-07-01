//! This module contains everything related to errors occurring in the gui.

use std::{fmt::Display, sync::PoisonError};

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
    /// At least one vault contains unsaved changes and cannot be closed.
    VaultContainsUnsavedChanges,
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

impl Display for PWDuckGuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PWDuckGuiError::Iced(error) => write!(f, "An error within Iced occurred: {}.", error),
            PWDuckGuiError::Mutex(error) => write!(f, "Could not lock a mutex: {}.", error),
            PWDuckGuiError::Option => write!(f, "Expected a value but there was none"),
            //PWDuckGuiError::PWDuckCoreError(error) => write!(f, "An error in the core occurred: {}.", error),
            PWDuckGuiError::PWDuckCoreError(error) => write!(f, "{}.", error),
            PWDuckGuiError::Unreachable(error) => write!(f, "An unreachable path was reached in: {}.", error),
            PWDuckGuiError::VaultContainsUnsavedChanges => write!(f, "Your vault contains unsaved changes. You have to save it before you are able to close it."),
        }
    }
}