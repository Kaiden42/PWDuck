//! TODO

/// TODO
#[derive(Debug)]
pub enum PWDuckGuiError {
    /// TODO
    Iced(iced::Error),
}

impl From<iced::Error> for PWDuckGuiError {
    fn from(error: iced::Error) -> Self {
        Self::Iced(error)
    }
}

/// TODO
#[derive(Clone, Debug)]
pub enum NfdError {
    /// TODO
    Null,
}
