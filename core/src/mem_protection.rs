//! TODO

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use zeroize::Zeroize;

use seckey::SecBytes;

use crate::{cryptography::fill_random_bytes, error::PWDuckCoreError};

#[cfg(test)]
use mocktopus::macros::*;

/// The size of a 1 MiB block.
const MIB_1: usize = 0x0010_0000;

/// Memory key used for in memory encryption. It is protected and locked in memory.
#[derive(Debug)]
pub struct MemKey {
    /// The bytes of the memory key.
    key: SecBytes,
}

impl Zeroize for MemKey {
    fn zeroize(&mut self) {
        self.key.write().zeroize();
    }
}

impl Drop for MemKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg_attr(test, mockable)]
impl MemKey {
    /// Create a new random memory key with the default size.
    #[must_use]
    pub fn new() -> Self {
        Self::with_length(MIB_1)
    }

    /// Create a new random memory key with the given size.
    #[must_use]
    pub fn with_length(length: usize) -> Self {
        Self {
            key: SecBytes::with(length, |buf| {
                #[cfg(not(debug_assertions))]
                fill_random_bytes(buf);
                #[cfg(debug_assertions)] // TODO
                buf.iter_mut()
                    .enumerate()
                    .for_each(|(i, x)| *x = (i % 16) as u8);
                //buf.iter_mut().for_each(|x| *x = 0xff);
            }),
        }
    }
}

impl From<SecBytes> for MemKey {
    fn from(bytes: SecBytes) -> Self {
        Self { key: bytes }
    }
}

impl Default for MemKey {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for MemKey {
    type Target = SecBytes;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl DerefMut for MemKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.key
    }
}

/// Wrapper around a [`Vec`](Vec). It zeroizes itself automatically at drop.
#[derive(PartialEq, Eq, Zeroize)]
//#[zeroize(drop)]
#[allow(missing_debug_implementations)]
#[cfg_attr(test, derive(Debug))]
pub struct SecVec<T: Zeroize>(Vec<T>);

#[cfg_attr(test, mockable)]
impl<T: Zeroize> SecVec<T> {
    /// Create a new empty [`SecVec`](SecVec).
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new [`SecVec`](SecVec) with the given capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

impl<T: Zeroize> Default for SecVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Zeroize> Deref for SecVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Zeroize> DerefMut for SecVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Zeroize> Drop for SecVec<T> {
    fn drop(&mut self) {
        //self.iter_mut().for_each(Zeroize::zeroize);
        self.0.zeroize();
    }
}

impl<T: Zeroize> From<Vec<T>> for SecVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

/// Wrapper around a [`String`](String). It zeroizes itself automatically at drop.
#[derive(Clone, Default, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
#[allow(missing_debug_implementations)]
pub struct SecString(String);

impl SecString {
    /// Create a new empty [`SecString`](SecString).
    #[must_use]
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Create a new [`SecString`](SecString) from a [`SecVec`](SecVec) containing
    /// UTF-8 encoded text.
    pub fn from_utf8(v: SecVec<u8>) -> Result<Self, PWDuckCoreError> {
        let raw = v.to_vec();
        drop(v);
        let result = String::from_utf8(raw);

        match result {
            Ok(s) => Ok(s.into()),
            Err(err) => {
                // In case of error => zeroize memory
                err.into_bytes().zeroize();

                Err(PWDuckCoreError::Error(
                    "Failed to convert bytes to string".into(),
                ))
            }
        }
    }
}

impl Deref for SecString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SecString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for SecString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecString {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl Debug for SecString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("This is a SecString")
    }
}

impl From<SecString> for String {
    fn from(sec_string: SecString) -> Self {
        sec_string.0.clone()
    }
}

/// This function tries to configure the process to prevent the creation of a core dump once this process crashes.
/// This should work on all Unix/Linux systems. On windows this will just silently fail.
pub fn try_to_prevent_core_dump() -> Result<(), PWDuckCoreError> {
    #[cfg(not(windows))]
    rlimit::setrlimit(
        rlimit::Resource::CORE,
        rlimit::Rlim::from_usize(0),
        rlimit::Rlim::from_usize(0),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use mocktopus::mocking::*;

    use crate::{SecString, SecVec};

    use super::{MemKey, MIB_1};

    #[test]
    fn new_key() {
        MemKey::with_length.mock_safe(|len| {
            assert_eq!(len, MIB_1);
            MockResult::Continue((len,))
        });

        // New key with default size
        let mem_key = MemKey::new();
        let guard = mem_key.key.read();
        assert_eq!(guard.len(), MIB_1);

        MemKey::with_length.clear_mock();

        // New key with size of 1
        let mem_key = MemKey::with_length(42);
        let guard = mem_key.key.read();
        assert_eq!(guard.len(), 42)
    }

    #[test]
    fn new_secvec() {
        SecVec::<u8>::with_capacity.mock_safe(|cap| {
            assert_eq!(cap, 0);
            MockResult::Continue((cap,))
        });

        let sec_vec: SecVec<u8> = SecVec::new();
        assert_eq!(sec_vec.len(), 0);
        assert_eq!(sec_vec.capacity(), 0);
        assert!(sec_vec.is_empty());

        SecVec::<u8>::with_capacity.clear_mock();

        let sec_vec: SecVec<u8> = SecVec::with_capacity(42);
        assert_eq!(sec_vec.len(), 0);
        assert_eq!(sec_vec.capacity(), 42);
        assert!(sec_vec.is_empty());
    }

    #[test]
    fn new_sec_string() {
        let sec_string = SecString::new();
        assert_eq!(sec_string.len(), 0);
        assert!(sec_string.is_empty());

        let bytes: SecVec<u8> = vec![0x41, 0x42, 0x43, 0x44, 0x45].into();

        let string = SecString::from_utf8(bytes).expect("Should be a valid utf8 encoded string");

        assert_eq!(string, "ABCDE".into());

        let bytes: SecVec<u8> = vec![0x41, 0x42, 0xFF, 0x43, 0x44].into();

        let string =
            SecString::from_utf8(bytes).expect_err("Should not be a valid utf8 encoded string");
    }
}
