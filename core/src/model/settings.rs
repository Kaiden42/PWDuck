//! TODO

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

/// The settings of the application.
#[derive(Debug, Deserialize, Serialize, Getters, Setters)]
pub struct ApplicationSettings {
    /// The color theme of the application.
    #[getset(get = "pub", set = "pub")]
    theme: theme::Theme,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            theme: theme::Theme::Light,
        }
    }
}

/// TODO
pub mod theme {
    use serde::{Deserialize, Serialize};

    /// The color theme of the application.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
    pub enum Theme {
        /// Use the light theme.
        Light,
        /// Use the dark theme.
        Dark,
    }
}
