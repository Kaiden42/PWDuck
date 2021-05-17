//! TODO
use crypto::symmetriccipher::SymmetricCipherError;

/// TODO
#[derive(Debug)]
pub enum PWDuckCoreError {
    /// TODO
    Argon2(argon2::password_hash::Error),
    /// TODO
    Aes(SymmetricCipherError),
    /// TODO
    Base64(base64::DecodeError),
    /// TODO
    Error(String),
    /// TODO
    IO(std::io::Error),
    /// TODO
    Ron(ron::Error),
    /// TODO
    Utf8(std::string::FromUtf8Error),
}

impl Clone for PWDuckCoreError {
    fn clone(&self) -> Self {
        match self {
            Self::Argon2(error) => Self::Argon2(*error),
            Self::Aes(error) => Self::Aes(*error),
            Self::Base64(error) => Self::Base64(error.clone()),
            Self::Error(error) => Self::Error(error.clone()),
            Self::IO(error) => Self::Error(format!("'Cloned' IO Error: {:?}", error)),
            Self::Ron(error) => Self::Ron(error.clone()),
            Self::Utf8(error) => Self::Utf8(error.clone()),
        }
    }
}

impl From<argon2::password_hash::Error> for PWDuckCoreError {
    fn from(error: argon2::password_hash::Error) -> Self {
        Self::Argon2(error)
    }
}

impl From<SymmetricCipherError> for PWDuckCoreError {
    fn from(error: SymmetricCipherError) -> Self {
        Self::Aes(error)
    }
}

impl From<base64::DecodeError> for PWDuckCoreError {
    fn from(error: base64::DecodeError) -> Self {
        Self::Base64(error)
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
