//! This module contains everything related to errors occurring in the core.

use std::{fmt::Display, sync::PoisonError};
/// An error thrown in the core.
#[derive(Debug)]
pub enum PWDuckCoreError {
    /// Key derivation with Argon2 failed.
    Argon2(argon2::password_hash::Error),
    /// Encoding or decoding with Base64 failed.
    Base64(base64::DecodeError),
    /// Encrypting or Decrypting with AES failed.
    BlockMode(block_modes::BlockModeError),
    /// Error thrown by an invalid length for the AES IV.
    BlockModeIV(block_modes::InvalidKeyIvLength),
    /// A generic error.
    Error(String),
    /// Reading or writing a file failed.
    IO(std::io::Error),
    /// Locking a mutex failed.
    Mutex(String),
    /// Serializing or deserializing with RON failed.
    Ron(ron::Error),
    /// An error from the autotype sequence parser.
    SequenceParseError(SequenceParseError),
    /// Wrong UFT8 encoding.
    Utf8(std::string::FromUtf8Error),
}

impl Clone for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn clone(&self) -> Self {
        match self {
            Self::Argon2(error) => Self::Argon2(*error),
            Self::Base64(error) => Self::Base64(error.clone()),
            Self::BlockMode(error) => Self::BlockMode(*error),
            Self::BlockModeIV(error) => Self::BlockModeIV(*error),
            Self::Error(error) => Self::Error(error.clone()),
            Self::IO(error) => Self::Error(format!("{}", error)),
            Self::Mutex(error) => Self::Mutex(error.clone()),
            Self::Ron(error) => Self::Ron(error.clone()),
            Self::SequenceParseError(error) => Self::SequenceParseError(error.clone()),
            Self::Utf8(error) => Self::Utf8(error.clone()),
        }
    }
}

impl From<argon2::password_hash::Error> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: argon2::password_hash::Error) -> Self {
        Self::Argon2(error)
    }
}

impl From<base64::DecodeError> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: base64::DecodeError) -> Self {
        Self::Base64(error)
    }
}

impl From<block_modes::BlockModeError> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: block_modes::BlockModeError) -> Self {
        Self::BlockMode(error)
    }
}

impl From<block_modes::InvalidKeyIvLength> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: block_modes::InvalidKeyIvLength) -> Self {
        Self::BlockModeIV(error)
    }
}

impl From<String> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(s: String) -> Self {
        Self::Error(s)
    }
}

impl From<std::io::Error> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl<T> From<PoisonError<T>> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: PoisonError<T>) -> Self {
        Self::Mutex(format!("{:?}", error))
    }
}

impl From<ron::Error> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: ron::Error) -> Self {
        Self::Ron(error)
    }
}

impl From<SequenceParseError> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: SequenceParseError) -> Self {
        Self::SequenceParseError(error)
    }
}

impl From<std::string::FromUtf8Error> for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl Display for PWDuckCoreError {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PWDuckCoreError::Argon2(error) => write!(f, "Could not derive a key ({})", error),
            PWDuckCoreError::Base64(error) => {
                write!(f, "Not a valid Base64 encoded string ({})", error)
            }
            PWDuckCoreError::BlockMode(error) => write!(
                f,
                "Wrong password or your vault might be corrupted ({})",
                error
            ),
            PWDuckCoreError::BlockModeIV(error) => {
                write!(f, "Got a wrong size of the IV ({})", error)
            }
            PWDuckCoreError::Error(error) => write!(f, "{}", error),
            PWDuckCoreError::IO(error) => write!(f, "Could not access the vault ({})", error),
            PWDuckCoreError::Mutex(error) => write!(f, "Could not lock a mutex ({})", error),
            PWDuckCoreError::Ron(error) => write!(f, "Not a valid RON structure ({})", error),
            PWDuckCoreError::SequenceParseError(error) => write!(
                f,
                "An error at parsing the autotype sequence occurred ({})",
                error
            ),
            PWDuckCoreError::Utf8(error) => {
                write!(f, "The given data was no valid UTF-8 ({})", error)
            }
        }
    }
}

/// An error from parsing an autotype sequence.
#[derive(Clone, Debug)]
pub enum SequenceParseError {
    /// Parsed an invalid field.
    InvalidField(String),
    /// Parsed an invalid key.
    InvalidKey(String),
    /// A parsing error.
    ParseError(String),
}

impl<T> From<pest::error::Error<T>> for SequenceParseError
where
    T: std::fmt::Debug,
{
    #[cfg_attr(coverage, no_coverage)]
    fn from(error: pest::error::Error<T>) -> Self {
        Self::ParseError(format!("{:?}", error))
    }
}

impl Display for SequenceParseError {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SequenceParseError::InvalidField(field) => write!(f, "Invalid field: {}", field),
            SequenceParseError::InvalidKey(key) => write!(f, "Invadil key: {}", key),
            SequenceParseError::ParseError(error) => write!(f, "Incorrect sequence: {}", error),
        }
    }
}
