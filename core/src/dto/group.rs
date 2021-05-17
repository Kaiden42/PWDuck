//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};
/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct Group {
    /// TODO
    #[getset(get = "pub")]
    iv: String,

    /// TODO
    #[getset(get = "pub")]
    content: String,
}

impl Group {
    /// TODO
    pub const fn new(iv: String, content: String) -> Self {
        Self { iv, content }
    }
}
