//! TODO
use aes::Aes256;
use argon2::{
    password_hash::{Ident, Salt, SaltString},
    Argon2, PasswordHasher,
};
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use chacha20::cipher::{NewCipher, StreamCipher};
use chacha20::{ChaCha20, Key, Nonce};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_core::OsRng;
use zeroize::Zeroize;

use crate::{
    dto::master_key::MasterKey,
    error::PWDuckCoreError,
    mem_protection::{MemKey, SecVec},
};

#[cfg(test)]
use mocktopus::macros::*;

/// The length of the initialization vector for the AES encryption.
pub const AES_IV_LENGTH: usize = 16;
/// The length of the nonce for the `ChaCha20` encryption.
pub const CHACHA20_NONCE_LENGTH: usize = 12;
/// The default size of a master key.
pub const MASTER_KEY_SIZE: usize = 32;

/// Generate a new random initialization vector for the AES encryption.
pub fn generate_aes_iv() -> Vec<u8> {
    generate_iv(AES_IV_LENGTH)
}

/// Generate a new random nonce (number-used-once) for the `ChaCha20` encryption.
pub fn generate_chacha20_nonce() -> Vec<u8> {
    // TODO: nonce must be unique!
    generate_iv(CHACHA20_NONCE_LENGTH)
}

/// Generate a new random salt for the Argon2 key derivation.
pub fn generate_argon2_salt() -> String {
    // TODO
    SaltString::generate(&mut OsRng).as_str().to_owned()
}

/// Generate a new random iv with the given length.
#[cfg_attr(test, mockable)]
pub fn generate_iv(length: usize) -> Vec<u8> {
    let mut iv: Vec<u8> = vec![0_u8; length];
    //OsRng.fill_bytes(&mut iv);
    fill_random_bytes(&mut iv);
    iv
}

/// Generate a new random salt.
#[cfg_attr(test, mockable)]
fn generate_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}

/// Fill the given slice of bytes with random values.
#[cfg_attr(test, mockable)]
pub fn fill_random_bytes(buf: &mut [u8]) {
    //let mut iv = vec![0u8; length];
    let mut rng = ChaCha20Rng::from_entropy();
    rng.fill_bytes(buf);
}

/// Hash the password.
#[cfg_attr(test, mockable)]
pub fn hash_password(password: &str, salt: &str) -> Result<SecVec<u8>, PWDuckCoreError> {
    derive_key(password.as_bytes(), salt)
}

/// Derive a memory key.
#[cfg_attr(test, mockable)]
pub fn derive_key_protection(mem_key: &MemKey, salt: &str) -> Result<SecVec<u8>, PWDuckCoreError> {
    derive_key(&mem_key.read(), salt)
}

/// Derive a key from date based on the given salt.
#[cfg_attr(test, mockable)]
pub fn derive_key(data: &[u8], salt: &str) -> Result<SecVec<u8>, PWDuckCoreError> {
    let password_hash = Argon2::default().hash_password(
        data,
        Some(Ident::new("argon2id")),
        argon2::Params::default(),
        Salt::new(salt)?,
    )?;
    // TODO: find a way to zeroize this...
    let password_hash = password_hash.hash.expect("There should be a hash");
    Ok(password_hash
        .as_bytes()
        .iter()
        .copied()
        .collect::<Vec<u8>>()
        .into())
}

/// Generate a new masterkey which will be encrypted with the given password after creation.
#[cfg_attr(test, mockable)]
pub fn generate_masterkey(password: &str) -> Result<MasterKey, PWDuckCoreError> {
    // Generate random salt
    let salt = generate_salt();

    // Hash password with KDF
    let password_hash = hash_password(password, salt.as_str())?;

    // Generate random initialization vector
    //let mut iv = [0u8; 16];
    //OsRng.fill_bytes(&mut iv);
    let iv = generate_aes_iv();

    // Generate random master key and encrypt it with password hash
    let mut master_key = [0_u8; MASTER_KEY_SIZE];
    #[cfg(not(debug_assertions))]
    fill_random_bytes(&mut master_key);
    //OsRng.fill_bytes(&mut master_key);
    #[cfg(debug_assertions)] // TODO
    master_key
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = (i % 16) as u8);

    let encrypted_key = aes_cbc_encrypt(&master_key, password_hash.as_slice(), &iv)?;
    master_key.zeroize();

    Ok(MasterKey::new(
        //salt: base64::encode(salt.as_bytes()),
        salt.as_str().to_owned(),
        base64::encode(iv),
        base64::encode(encrypted_key),
    ))
}

/// Decrypt a masterkey with the given password and encrypt it with the given memory key.
#[cfg_attr(test, mockable)]
pub fn decrypt_masterkey(
    masterkey: &MasterKey,
    password: &str,
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<crate::model::master_key::MasterKey, PWDuckCoreError> {
    let mut hash = hash_password(password, masterkey.salt())?;

    let encrypted_key = base64::decode(masterkey.encrypted_key())?;
    let mut iv = base64::decode(masterkey.iv())?;

    // unprotected key
    let mut key = aes_cbc_decrypt(&encrypted_key, &hash, &iv)?;
    hash.zeroize();
    iv.zeroize();

    // protect key
    let protected_key = protect_masterkey(&key, key_protection, nonce)?;
    key.zeroize();

    Ok(protected_key.into())
}

/// Protect the masterkey by encrypting it with the given key.
#[cfg_attr(test, mockable)]
pub fn protect_masterkey(
    master_key: &[u8],
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, PWDuckCoreError> {
    chacha20_encrypt(master_key, key_protection, nonce)
}

/// Unprotect the masterkey by decrypting it with the given key.
#[cfg_attr(test, mockable)]
pub fn unprotect_masterkey(
    master_key: &[u8],
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<SecVec<u8>, PWDuckCoreError> {
    chacha20_decrypt(master_key, key_protection, nonce)
}

/// Encrypt the data with the AES block cipher in CBC mode.
///
/// It expects:
///     - The data to encrypt
///     - The key for the encryption
///     - The iv for the CBC mode
#[cfg_attr(test, mockable)]
pub fn aes_cbc_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, PWDuckCoreError> {
    let cipher = Cbc::<Aes256, Pkcs7>::new_from_slices(key, iv)?;
    Ok(cipher.encrypt_vec(data))
}

/// Decrypt the data with the AES block cipher in CBC mode.
///
/// It expects:
///     - The data to decrypt
///     - The key for the decryption
///     - The iv for the CBC mode
#[cfg_attr(test, mockable)]
pub fn aes_cbc_decrypt(
    encrypted_data: &[u8],
    key: &[u8],
    iv: &[u8],
) -> Result<SecVec<u8>, PWDuckCoreError> {
    let cipher = Cbc::<Aes256, Pkcs7>::new_from_slices(key, iv)?;
    Ok(cipher.decrypt_vec(encrypted_data)?.into())
}

/// Encrypt the data with the `ChaCHa20` stream cipher.
///
/// It expects:
///     - The data to encrypt
///     - The key for the encryption
///     - A nonce (number-used-once)
#[allow(clippy::unnecessary_wraps)]
#[cfg_attr(test, mockable)]
pub fn chacha20_encrypt(data: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, PWDuckCoreError> {
    let mut cipher = ChaCha20::new(Key::from_slice(key), Nonce::from_slice(nonce));

    let mut result = data.to_vec();
    cipher.apply_keystream(&mut result);

    Ok(result)
}

/// Decrypt the data with the `ChaCha20` stream cipher.
///
/// It expects:
///     - The data to decrypt
///     - The key for the decryption
///     - A nonce (number-used-once)
#[allow(clippy::unnecessary_wraps)]
#[cfg_attr(test, mockable)]
pub fn chacha20_decrypt(
    encrypted_data: &[u8],
    key: &[u8],
    nonce: &[u8],
) -> Result<SecVec<u8>, PWDuckCoreError> {
    let mut cipher = ChaCha20::new(Key::from_slice(key), Nonce::from_slice(nonce));

    let mut result = encrypted_data.to_vec();
    cipher.apply_keystream(&mut result);

    Ok(result.into())
}

#[cfg(test)]
mod tests {
    use argon2::password_hash::SaltString;
    use rand_core::OsRng;
    use seckey::SecBytes;

    use crate::cryptography::{protect_masterkey, unprotect_masterkey, MASTER_KEY_SIZE};
    use crate::mem_protection::MemKey;
    use crate::{PWDuckCoreError, SecVec};

    use super::derive_key;
    use super::generate_masterkey;
    use super::hash_password;
    use super::{
        aes_cbc_decrypt, aes_cbc_encrypt, chacha20_decrypt, chacha20_encrypt, decrypt_masterkey,
        derive_key_protection, fill_random_bytes, generate_aes_iv, generate_chacha20_nonce,
        generate_iv, generate_salt,
    };
    use super::{AES_IV_LENGTH, CHACHA20_NONCE_LENGTH};

    // TODO: mocking
    use mocktopus::mocking::*;

    const PASSWORD: &'static str = "This is a totally secret password";
    const SALT: &'static str = "pa7lMD/slzor2CVNHZWNyA";

    #[test]
    fn test_mocking() {
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(0);
            MockResult::Return(())
        });
        let mut buf: [u8; 4] = [1u8; 4];
        fill_random_bytes(&mut buf);
        assert_eq!([0, 0, 0, 0], buf);

        let iv = generate_aes_iv();
        let expected = [0u8; AES_IV_LENGTH];
        assert_eq!(iv, expected);
    }

    #[test]
    fn test_generate_aes_iv() {
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(42);
            MockResult::Return(())
        });

        let iv = generate_aes_iv();

        assert!(!iv.is_empty());
        assert_eq!(iv.len(), AES_IV_LENGTH);
        assert_eq!(iv, vec![42_u8; AES_IV_LENGTH]);
    }

    #[test]
    fn test_generate_chacha20_nonce() {
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(21);
            MockResult::Return(())
        });

        let nonce = generate_chacha20_nonce();

        assert!(!nonce.is_empty());
        assert_eq!(nonce.len(), CHACHA20_NONCE_LENGTH);
        assert_eq!(nonce, vec![21_u8; CHACHA20_NONCE_LENGTH]);
    }

    #[test]
    fn test_fill_random_bytes() {
        let mut bytes1 = [0u8; 16];
        fill_random_bytes(&mut bytes1);

        assert_ne!(bytes1, [0u8; 16]);

        let mut bytes2 = [0u8; 16];
        fill_random_bytes(&mut bytes2);

        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_hash_password() {
        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);

        let hash1 =
            hash_password(PASSWORD, salt1.as_str()).expect("Hashing passwords should not fail");
        assert_ne!(hash1.as_slice(), PASSWORD.as_bytes());

        let hash2 =
            hash_password(PASSWORD, salt2.as_str()).expect("Hashing passwords should not fail");
        assert_ne!(hash1, hash2);

        let hash1_again =
            hash_password(PASSWORD, salt1.as_str()).expect("Hashing passwords should not fail");
        assert_eq!(hash1, hash1_again);
    }

    #[test]
    fn test_derive_key_protection() {
        MemKey::with_length.mock_safe(|length| {
            MockResult::Return(SecBytes::with(length, |buf| buf.fill(21_u8)).into())
        });
        derive_key.mock_safe(|data, salt| {
            assert_eq!(salt, SALT);
            if data.iter().all(|b| *b == 21u8) {
                MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_SIZE].into()))
            } else {
                MockResult::Return(Err(PWDuckCoreError::Error(
                    "Not the expected MemKey".into(),
                )))
            }
        });

        let mem_key = MemKey::new();

        let key = derive_key_protection(&mem_key, SALT).expect("Deriving key should not fail");
        let expected: SecVec<u8> = vec![42_u8; argon2::Params::DEFAULT_OUTPUT_SIZE].into();

        assert_eq!(key, expected);
    }

    #[test]
    fn test_derive_key() {
        let key = derive_key(PASSWORD.as_bytes(), SALT).expect("Foo");

        let expected: SecVec<u8> = vec![
            72, 219, 9, 132, 177, 130, 185, 39, 90, 221, 173, 231, 171, 35, 7, 161, 205, 33, 148,
            192, 113, 22, 241, 202, 219, 231, 171, 134, 19, 56, 183, 152,
        ]
        .into();

        assert_eq!(key, expected);
    }

    #[test]
    fn test_generate_masterkey() {
        let key1 = generate_masterkey(PASSWORD).expect("Generating masterkey should not fail");
        let key2 = generate_masterkey(PASSWORD).expect("Generating masterkey should not fail");

        assert_ne!(key1.salt(), key2.salt());
        assert_ne!(key1.iv(), key2.iv());
        assert_ne!(key1.encrypted_key(), key2.encrypted_key());

        let decrypted_key1 = aes_cbc_decrypt(
            &base64::decode(key1.encrypted_key()).unwrap(),
            &hash_password(PASSWORD, key1.salt()).unwrap(),
            &base64::decode(key1.iv()).unwrap(),
        )
        .unwrap();

        let decrypted_key2 = aes_cbc_decrypt(
            &base64::decode(key2.encrypted_key()).unwrap(),
            &hash_password(PASSWORD, key2.salt()).unwrap(),
            &base64::decode(key2.iv()).unwrap(),
        )
        .unwrap();

        assert_ne!(decrypted_key1, decrypted_key2);

        generate_salt.mock_safe(|| MockResult::Return(SaltString::new(SALT).unwrap()));
        hash_password.mock_safe(|pwd, salt| {
            assert_eq!(pwd, PASSWORD);
            assert_eq!(salt, SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_SIZE].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let master_key =
            generate_masterkey(PASSWORD).expect("Generating masterkey should not fail");

        let dercypted_key = aes_cbc_decrypt(
            &base64::decode(master_key.encrypted_key()).unwrap(),
            &hash_password(PASSWORD, master_key.salt()).unwrap(),
            &base64::decode(master_key.iv()).unwrap(),
        )
        .unwrap();

        let expected: SecVec<u8> = vec![255_u8; MASTER_KEY_SIZE].into();

        assert_eq!(dercypted_key, expected);
    }

    #[test]
    fn test_decrypt_masterkey() {
        generate_salt.mock_safe(|| MockResult::Return(SaltString::new(SALT).unwrap()));
        hash_password.mock_safe(|password, salt| {
            assert_eq!(password, PASSWORD);
            assert_eq!(salt, SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_SIZE].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let masterkey = generate_masterkey(PASSWORD).unwrap();

        MemKey::with_length.mock_safe(|length| {
            MockResult::Return(SecBytes::with(length, |buf| buf.fill(21_u8)).into())
        });

        let mem_key = MemKey::new();

        let salt = generate_salt();
        let nonce = generate_chacha20_nonce();

        let key_protection = derive_key_protection(&mem_key, salt.as_str()).unwrap();
        unsafe {
            aes_cbc_decrypt.mock_raw(|encrypted_key, hash, iv| {
                assert_eq!(
                    encrypted_key,
                    base64::decode(masterkey.encrypted_key()).unwrap()
                );
                assert_eq!(hash, &[42_u8; argon2::Params::DEFAULT_OUTPUT_SIZE]);
                assert_eq!(iv, vec![21_u8; AES_IV_LENGTH]);
                MockResult::Continue((encrypted_key, hash, iv))
            });
        }

        unsafe {
            protect_masterkey.mock_raw(|key, kp, no| {
                assert_eq!(
                    key,
                    aes_cbc_decrypt(
                        &base64::decode(masterkey.encrypted_key()).unwrap(),
                        &hash_password(PASSWORD, masterkey.salt()).unwrap(),
                        &base64::decode(masterkey.iv()).unwrap(),
                    )
                    .unwrap()
                    .as_slice(),
                );
                assert_eq!(kp, key_protection.as_slice());
                assert_eq!(no, nonce);
                MockResult::Continue((key, kp, no))
            });
        }

        let decrypted_key = decrypt_masterkey(&masterkey, PASSWORD, &key_protection, &nonce)
            .expect("Decrypting masterkey should not fail.");

        let unprotected_key =
            unprotect_masterkey(decrypted_key.as_slice(), &key_protection, &nonce)
                .expect("Unprotecting masterkey should not fail");

        assert_eq!(
            unprotected_key,
            aes_cbc_decrypt(
                &base64::decode(masterkey.encrypted_key()).unwrap(),
                &hash_password(PASSWORD, masterkey.salt()).unwrap(),
                &base64::decode(masterkey.iv()).unwrap(),
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_protect_masterkey() {
        let master_key = [255_u8; MASTER_KEY_SIZE];
        let key_protection = [42u8; argon2::Params::DEFAULT_OUTPUT_SIZE];
        let nonce = [21_u8; CHACHA20_NONCE_LENGTH];

        unsafe {
            chacha20_encrypt.mock_raw(|m, k, n| {
                assert_eq!(m, master_key);
                assert_eq!(k, key_protection);
                assert_eq!(n, nonce);
                MockResult::Return(Ok(vec![84_u8; MASTER_KEY_SIZE]))
            });
        }

        let protected_key = protect_masterkey(&master_key, &key_protection, &nonce)
            .expect("Protecting masterkey should not fail");
        assert_eq!(protected_key, vec![84_u8; MASTER_KEY_SIZE]);
    }

    #[test]
    fn test_unprotect_masterkey() {
        let master_key = [84_u8; MASTER_KEY_SIZE];
        let key_protection = [42_u8; argon2::Params::DEFAULT_OUTPUT_SIZE];
        let nonce = [21_u8; CHACHA20_NONCE_LENGTH];

        unsafe {
            chacha20_decrypt.mock_raw(|m, k, n| {
                assert_eq!(m, master_key);
                assert_eq!(k, key_protection);
                assert_eq!(n, nonce);
                MockResult::Return(Ok(vec![255_u8; MASTER_KEY_SIZE].into()))
            });
        }

        let unprotected_key = unprotect_masterkey(&master_key, &key_protection, &nonce)
            .expect("Unprotecting masterkey should not fail");
        assert_eq!(unprotected_key, vec![255_u8; MASTER_KEY_SIZE].into())
    }

    #[test]
    fn test_aes_cbc_encrypt_decrypt() {
        let data = "This is the data";
        let encrypted = aes_cbc_encrypt(data.as_bytes(), &[1u8; 32], &[1u8; AES_IV_LENGTH])
            .expect("AES should be able to encrypt some data");
        assert!(!encrypted.is_empty());
        assert_ne!(data.as_bytes(), encrypted.as_slice());

        let decrypted = aes_cbc_decrypt(encrypted.as_slice(), &[1u8; 32], &[1u8; AES_IV_LENGTH])
            .expect("AES should be able to decrypt some data");
        assert!(!decrypted.is_empty());
        assert_ne!(decrypted.as_slice(), encrypted.as_slice());

        assert_eq!(decrypted.as_slice(), data.as_bytes());
    }

    #[test]
    fn test_chacha20_encrypt_decrypt() {
        let data = "This is the data";
        let encrypted =
            chacha20_encrypt(data.as_bytes(), &[1u8; 32], &[1u8; CHACHA20_NONCE_LENGTH])
                .expect("ChaCha20 should be able to encrypt some data");
        assert!(!encrypted.is_empty());
        assert_ne!(data.as_bytes(), encrypted.as_slice());

        let decrypted = chacha20_decrypt(
            encrypted.as_slice(),
            &[1u8; 32],
            &[1u8; CHACHA20_NONCE_LENGTH],
        )
        .expect("ChaCha20 should be able to decrypt some data");
        assert!(!decrypted.is_empty());
        assert_ne!(decrypted.as_slice(), encrypted.as_slice());

        assert_eq!(decrypted.as_slice(), data.as_bytes());
    }
}
