//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct EntryHead {
    /// TODO
    #[getset(get = "pub")]
    iv: String,

    /// tODO
    #[getset(get = "pub")]
    content: String,
}

impl EntryHead {
    /// TODO
    pub const fn new(iv: String, head: String) -> Self {
        Self { iv, content: head }
    }
}

/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct EntryBody {
    /// TODO
    #[getset(get = "pub")]
    iv: String,

    /// TODO
    #[getset(get = "pub")]
    content: String,
}

impl EntryBody {
    /// TODO
    pub const fn new(iv: String, body: String) -> Self {
        Self { iv, content: body }
    }
}
