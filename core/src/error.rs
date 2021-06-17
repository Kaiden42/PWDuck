//! TODO

use std::sync::PoisonError;
/// TODO
#[derive(Debug)]
pub enum PWDuckCoreError {
    /// TODO
    Argon2(argon2::password_hash::Error),
    /// TODO
    /// TODO
    Base64(base64::DecodeError),
    /// TODO
    BlockMode(block_modes::BlockModeError),
    /// TODO
    BlockModeIV(block_modes::InvalidKeyIvLength),
    /// TODO
    Error(String),
    /// TODO
    IO(std::io::Error),
    /// TODO
    Mutex(String),
    /// TODO
    Ron(ron::Error),
    /// TODO
    Utf8(std::string::FromUtf8Error),
    //// TODO
    //ZxcvbnError(zxcvbn::ZxcvbnError),
}

impl Clone for PWDuckCoreError {
    fn clone(&self) -> Self {
        match self {
            Self::Argon2(error) => Self::Argon2(*error),
            Self::Base64(error) => Self::Base64(error.clone()),
            Self::BlockMode(error) => Self::BlockMode(*error),
            Self::BlockModeIV(error) => Self::BlockModeIV(*error),
            Self::Error(error) => Self::Error(error.clone()),
            Self::IO(error) => Self::Error(format!("'Cloned' IO Error: {:?}", error)),
            Self::Mutex(error) => Self::Mutex(error.clone()),
            Self::Ron(error) => Self::Ron(error.clone()),
            Self::Utf8(error) => Self::Utf8(error.clone()),
            //Self::ZxcvbnError(error) => Self::ZxcvbnError(error.clone()),
        }
    }
}

impl From<argon2::password_hash::Error> for PWDuckCoreError {
    fn from(error: argon2::password_hash::Error) -> Self {
        Self::Argon2(error)
    }
}

impl From<base64::DecodeError> for PWDuckCoreError {
    fn from(error: base64::DecodeError) -> Self {
        Self::Base64(error)
    }
}

impl From<block_modes::BlockModeError> for PWDuckCoreError {
    fn from(error: block_modes::BlockModeError) -> Self {
        Self::BlockMode(error)
    }
}

impl From<block_modes::InvalidKeyIvLength> for PWDuckCoreError {
    fn from(error: block_modes::InvalidKeyIvLength) -> Self {
        Self::BlockModeIV(error)
    }
}

impl From<String> for PWDuckCoreError {
    fn from(s: String) -> Self {
        Self::Error(s)
    }
}

impl From<std::io::Error> for PWDuckCoreError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl<T> From<PoisonError<T>> for PWDuckCoreError {
    fn from(error: PoisonError<T>) -> Self {
        Self::Mutex(format!("{:?}", error))
    }
}

impl From<ron::Error> for PWDuckCoreError {
    fn from(error: ron::Error) -> Self {
        Self::Ron(error)
    }
}

impl From<std::string::FromUtf8Error> for PWDuckCoreError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::Utf8(error)
    }
}

/*impl From<zxcvbn::ZxcvbnError> for PWDuckCoreError {
    fn from(error: zxcvbn::ZxcvbnError) -> Self {
        Self::ZxcvbnError(error)
    }
}*/
