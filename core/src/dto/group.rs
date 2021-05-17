//! TODO
use getset::Getters;
use serde::{Deserialize, Serialize};
/// TODO
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct Group {
    #[getset(get = "pub")]
    iv: String,

    #[getset(get = "pub")]
    content: String,
}

impl Group {
    /// TODO
    pub fn new(iv: String, content: String) -> Self {
        Self { iv, content }
    }
}
