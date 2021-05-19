//! TODO

use std::sync::PoisonError;

use pwduck_core::PWDuckCoreError;

/// TODO
#[derive(Debug)]
pub enum PWDuckGuiError {
    /// TODO
    Iced(iced::Error),
    /// TODO
    Mutex(String),
    /// TODO
    Option,
    /// TODO
    PWDuckCoreError(PWDuckCoreError),
    /// TODO
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

/// TODO
#[derive(Clone, Debug)]
pub enum NfdError {
    /// TODO
    Null,
}

impl<T> From<PWDuckGuiError> for Result<T, PWDuckGuiError> {
    fn from(error: PWDuckGuiError) -> Self {
        Err(error)
    }
}
