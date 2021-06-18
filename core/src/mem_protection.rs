//! TODO

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use zeroize::Zeroize;

use seckey::SecBytes;

use crate::{cryptography::fill_random_bytes, error::PWDuckCoreError};

/// TODO
const MIB_1: usize = 0x0010_0000;

/// TODO
#[derive(Debug)]
pub struct MemKey {
    /// TODO
    key: SecBytes,
}

impl Zeroize for MemKey {
    fn zeroize(&mut self) {
        self.key.write().zeroize();
    }
}

impl Drop for MemKey {
    fn drop(&mut self) {
        self.zeroize()
    }
}

impl MemKey {
    /// TODO
    #[must_use]
    pub fn new() -> Self {
        Self::with_length(MIB_1)
    }

    /// TODO
    #[must_use]
    pub fn with_length(length: usize) -> Self {
        Self {
            key: SecBytes::with(length, |buf| {
                /*use rand::prelude::*;
                use rand_chacha::ChaCha20Rng;

                let mut rng = ChaCha20Rng::from_entropy();
                rng.fill_bytes(buf)*/
                fill_random_bytes(buf);
            }),
        }
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

/// TODO
#[derive(PartialEq, Eq, Zeroize)]
//#[zeroize(drop)]
#[allow(missing_debug_implementations)]
#[cfg_attr(test, derive(Debug))]
pub struct SecVec<T: Zeroize>(Vec<T>);

impl<T: Zeroize> SecVec<T> {
    /// TODO
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// TODO
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

/// TODO
#[derive(Clone, Default, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
#[allow(missing_debug_implementations)]
pub struct SecString(String);

impl SecString {
    /// TODO
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

#[cfg(test)]
mod tests {
    use super::{MemKey, MIB_1};

    #[test]
    fn new_key() {
        // New key with default size
        let mem_key = MemKey::new();
        let guard = mem_key.key.read();
        assert_eq!(guard.len(), MIB_1);

        // New key with size of 1
        let mem_key = MemKey::with_length(1);
        let guard = mem_key.key.read();
        assert_eq!(guard.len(), 1)
    }
}
