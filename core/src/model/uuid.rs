//! TODO

use std::path::Path;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::io::generate_uuid;
/// TODO
#[derive(Debug, Deserialize, Serialize, Zeroize)]
pub struct Uuid {
    id: Vec<u8>,
}

impl Uuid {
    /// TODO
    pub fn new(path: &Path) -> Self {
        generate_uuid(path)
    }

    /// TODO
    pub fn as_string(&self) -> String {
        base64::encode(sha256::digest_bytes(&self.id))
    }
}

impl From<Vec<u8>> for Uuid {
    fn from(v: Vec<u8>) -> Self {
        Self { id: v }
    }
}
