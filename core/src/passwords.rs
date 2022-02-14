//! Password generation.
use rand::prelude::{Distribution, SliceRandom};
use rand_core::SeedableRng;

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
        self.0.extend_from_slice(chars);
    }
}

impl std::ops::Deref for Symbols {
    type Target = Vec<char>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Distribution<char> for Symbols {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> char {
        self.0.choose(rng).copied().unwrap_or('?') as char
    }
}

/// Generate a new password based on the given character pool of symbols with the specified length.
#[must_use]
pub fn generate_password(length: u8, symbols: &Symbols) -> String {
    use rand::Rng;
    use rand_chacha::ChaCha20Rng;

    let mut rng = ChaCha20Rng::from_entropy();

    let password: String = std::iter::repeat(())
        .map(|_| rng.sample(&symbols))
        .map(char::from)
        .take(length as usize)
        .collect();

    password
}

/// Calculate the entropy of the given password.
#[cfg_attr(coverage, no_coverage)]
#[must_use]
pub fn password_entropy(password: &str) -> pw_entropy::PasswordInfo {
    pw_entropy::PasswordInfo::for_password(password)
}

#[cfg(test)]
mod tests {
    use crate::{generate_password, Symbols};

    #[test]
    fn new_symbols() {
        let symbols = Symbols::new();
        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn append_symbols() {
        let mut symbols = Symbols::new();

        symbols.append(&Symbols::LOWER_ALPHA);

        assert!(contains_sub_slice(&symbols, &Symbols::LOWER_ALPHA));
        assert!(!contains_sub_slice(&symbols, &Symbols::UPPER_ALPHA));
        assert!(!contains_sub_slice(&symbols, &Symbols::NUMBERS));
        assert!(!contains_sub_slice(&symbols, &Symbols::SPECIAL));

        symbols.append(&Symbols::SPECIAL);

        assert!(contains_sub_slice(&symbols, &Symbols::LOWER_ALPHA));
        assert!(!contains_sub_slice(&symbols, &Symbols::UPPER_ALPHA));
        assert!(!contains_sub_slice(&symbols, &Symbols::NUMBERS));
        assert!(contains_sub_slice(&symbols, &Symbols::SPECIAL));
    }

    fn contains_sub_slice<T: std::cmp::PartialEq>(vec: &Vec<T>, slice: &[T]) -> bool {
        let mut found = false;
        for window in vec.windows(slice.len()) {
            if window == slice {
                found = true;
            }
        }
        found
    }

    #[test]
    fn password_generation() {
        let mut symbols = Symbols::new();

        symbols.append(&Symbols::LOWER_ALPHA);

        let password = generate_password(32, &symbols);
        assert_eq!(password.len(), 32);

        assert!(password.chars().into_iter().all(|c| symbols.contains(&c)));
        assert!(!password
            .chars()
            .into_iter()
            .any(|c| Symbols::UPPER_ALPHA.contains(&c)));
        assert!(!password
            .chars()
            .into_iter()
            .any(|c| Symbols::NUMBERS.contains(&c)));
        assert!(!password
            .chars()
            .into_iter()
            .any(|c| Symbols::SPECIAL.contains(&c)));

        let mut symbols = Symbols::new();
        symbols.append(&Symbols::NUMBERS);
        symbols.append(&Symbols::SPECIAL);

        let password = generate_password(64, &symbols);
        assert_eq!(password.len(), 64);

        assert!(password.chars().into_iter().all(|c| symbols.contains(&c)));
        assert!(!password
            .chars()
            .into_iter()
            .any(|c| Symbols::LOWER_ALPHA.contains(&c)));
        assert!(!password
            .chars()
            .into_iter()
            .any(|c| Symbols::UPPER_ALPHA.contains(&c)));
    }
}
