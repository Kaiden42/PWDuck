//! The data-transfer-objects of the entries.
//!
//! They represent the encrypted [`EntryHead`](crate::model::entry::EntryHead)
//! and [`EntryBody`](crate::model::entry::EntryBody) that are stored on disk.
use getset::Getters;
use serde::{Deserialize, Serialize};

/// The encrypted head of an entry as a data-transfer-object (dto).
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct EntryHead {
    /// The iv used to encrypt this entry head.
    #[getset(get = "pub")]
    iv: String,

    /// The encrypted content of this entry head.
    #[getset(get = "pub")]
    content: String,
}

impl EntryHead {
    /// Create a new [`EntryHead`](EntryHead).
    pub const fn new(iv: String, head: String) -> Self {
        Self { iv, content: head }
    }
}

/// The encrypted body of an entry as a data-transfer-object (dto).
#[derive(Clone, Debug, Deserialize, Serialize, Getters)]
pub struct EntryBody {
    /// The iv used to encrypt this entry body.
    #[getset(get = "pub")]
    iv: String,

    /// The encrypted content of this entry body.
    #[getset(get = "pub")]
    content: String,
}

impl EntryBody {
    /// Create a new [`EntryBody`](EntryBody).
    pub const fn new(iv: String, body: String) -> Self {
        Self { iv, content: body }
    }
}

#[cfg(test)]
mod tests {

    use super::{EntryBody, EntryHead};

    #[test]
    fn new_head() {
        let head = EntryHead::new("IV".into(), "HEAD".into());
        assert_eq!(head.iv(), "IV");
        assert_eq!(head.content(), "HEAD");
    }

    #[test]
    fn new_body() {
        let body = EntryBody::new("IV".into(), "BODY".into());
        assert_eq!(body.iv(), "IV");
        assert_eq!(body.content(), "BODY");
    }
}
