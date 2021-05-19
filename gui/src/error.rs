//! TODO

use pwduck_core::PWDuckCoreError;

/// TODO
#[derive(Debug)]
pub enum PWDuckGuiError {
    /// TODO
    Iced(iced::Error),
    /// TODO
    PWDuckCoreError(PWDuckCoreError),
}

impl From<iced::Error> for PWDuckGuiError {
    fn from(error: iced::Error) -> Self {
        Self::Iced(error)
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
