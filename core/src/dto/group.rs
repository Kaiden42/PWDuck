//! TODO
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    iv: String,
    content: String,
}

impl Group {
    /// TODO
    pub fn new(iv: String, content: String) -> Self {
        Self { iv, content }
    }

    /// TODO
    pub fn get_iv(&self) -> &str {
        &self.iv
    }

    /// TODO
    pub fn get_content(&self) -> &str {
        &self.content
    }
}
