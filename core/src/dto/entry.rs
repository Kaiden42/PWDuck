//! TODO
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntryHead {
    iv: String,
    content: String,
}

impl EntryHead {
    /// TODO
    pub fn new(iv: String, head: String) -> Self {
        Self { iv, content: head }
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

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntryBody {
    iv: String,
    content: String,
}

impl EntryBody {
    /// TODO
    pub fn new(iv: String, body: String) -> Self {
        Self { iv, content: body }
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
