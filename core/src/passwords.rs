//! TODO

use rand::prelude::{Distribution, SliceRandom};

use crate::PWDuckCoreError;

/// Character pool for password generation.
#[derive(Debug, Default)]
pub struct Symbols(Vec<char>);

impl Symbols {
    /// Lowercase latin alpha characters.
    pub const LOWER_ALPHA: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    /// Uppercase latin alpha characters
    pub const UPPER_ALPHA: [char; 26] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    /// Digits 0-9.
    pub const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

    /// Some special characters.
    pub const SPECIAL: [char; 28] = [
        '\\', '/', '{', '}', '*', '?', '(', ')', '-', ':', '@', '_', '[', ']', '^', '!', '<', '>',
        '=', '&', '#', '$', '|', '~', '`', '+', '%', ';',
    ];

    /// Create a new empty character pool.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Append characters to the pool.
    pub fn append(&mut self, chars: &[char]) {
        self.0.extend_from_slice(chars)
    }
}

impl Distribution<char> for Symbols {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> char {
        self.0.choose(rng).copied().unwrap_or('?') as char
    }
}

/// Generate a new password based on the given character pool of symbols with the specified length.
pub fn generate_password(length: u8, symbols: &Symbols) -> Result<String, PWDuckCoreError> {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let password: String = std::iter::repeat(())
        .map(|_| rng.sample(&symbols))
        .map(char::from)
        .take(length as usize)
        .collect();

    Ok(password)
}

/// Calculate the entropy of the given password.
pub fn password_entropy(password: &str) -> Result<pw_entropy::PasswordInfo, PWDuckCoreError> {
    Ok(pw_entropy::PasswordInfo::for_password(password))
}
