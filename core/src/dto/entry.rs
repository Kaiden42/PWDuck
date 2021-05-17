//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct EntryHead {
    #[getset(get = "pub")]
    iv: String,

    #[getset(get = "pub")]
    content: String,
}

impl EntryHead {
    /// TODO
    pub fn new(iv: String, head: String) -> Self {
        Self { iv, content: head }
    }
}

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct EntryBody {
    #[getset(get = "pub")]
    iv: String,
    #[getset(get = "pub")]
    content: String,
}

impl EntryBody {
    /// TODO
    pub fn new(iv: String, body: String) -> Self {
        Self { iv, content: body }
    }
}
