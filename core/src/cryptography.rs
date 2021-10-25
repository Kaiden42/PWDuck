//! A collection of all cryptographic functions.
use std::{collections::HashSet, path::Path, sync::Mutex};

use aes::Aes256;
use argon2::Argon2;
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use chacha20::cipher::{NewCipher, StreamCipher};
use chacha20::{ChaCha20, Key, Nonce};
use lazy_static::lazy_static;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use zeroize::Zeroize;

use crate::{
    dto::{key_file::KeyFile, master_key::MasterKey},
    error::PWDuckCoreError,
    mem_protection::{MemKey, SecVec},
};

#[cfg(test)]
use mocktopus::macros::*;

/// The length of the initialization vector for the AES encryption.
pub const AES_IV_LENGTH: usize = 16;
/// The length of the nonce for the `ChaCha20` encryption.
pub const CHACHA20_NONCE_LENGTH: usize = 12;
/// The length of the salt used for Argon2.
pub const SALT_LENGTH: usize = 16;
/// The default size of a master key.
pub const MASTER_KEY_SIZE: usize = 32;
/// The default size of a key file.
pub const KEY_FILE_SIZE: usize = 32;

lazy_static! {
    static ref USED_NONCE: Mutex<HashSet<Vec<u8>>> = Mutex::new(HashSet::new());
}

/// Generate a new random initialization vector for the AES encryption.
pub fn generate_aes_iv() -> Vec<u8> {
    generate_iv(AES_IV_LENGTH)
}

/// Generate a new random nonce (number-used-once) for the `ChaCha20` encryption.
pub fn generate_chacha20_nonce() -> Result<Vec<u8>, PWDuckCoreError> {
    #[allow(unused_mut)]
    let mut nonce;
    loop {
        nonce = generate_iv(CHACHA20_NONCE_LENGTH);

        // Retry if nonce is already in use
        #[cfg(not(test))]
        {
            let mut nonce_set = USED_NONCE.lock()?;
            if nonce_set.insert(nonce.clone()) {
                break;
            }
        }
        // Do not test if nonce is already in use in test environment
        #[cfg(test)]
        break;
    }
    Ok(nonce)
}

/// Generate a new random iv with the given length.
#[cfg_attr(test, mockable)]
pub fn generate_iv(length: usize) -> Vec<u8> {
    let mut iv: Vec<u8> = vec![0_u8; length];
    fill_random_bytes(&mut iv);
    iv
}

/// Generate a new random salt.
#[cfg_attr(test, mockable)]
#[cfg_attr(coverage, no_coverage)]
pub fn generate_salt() -> Vec<u8> {
    let mut iv: Vec<u8> = vec![0_u8; SALT_LENGTH];
    fill_random_bytes(&mut iv);
    iv
}

/// Fill the given slice of bytes with random values.
#[cfg_attr(test, mockable)]
pub fn fill_random_bytes(buf: &mut [u8]) {
    let mut rng = ChaCha20Rng::from_entropy();
    rng.fill_bytes(buf);
}

/// Hash the password.
#[cfg_attr(test, mockable)]
pub fn hash_password(password: &str, salt: &[u8]) -> Result<SecVec<u8>, PWDuckCoreError> {
    derive_key(password.as_bytes(), salt)
}

/// Derive a memory key.
#[cfg_attr(test, mockable)]
pub fn derive_key_protection(mem_key: &MemKey, salt: &[u8]) -> Result<SecVec<u8>, PWDuckCoreError> {
    derive_key(&mem_key.read(), salt)
}

/// Derive a key from date based on the given salt.
#[cfg_attr(test, mockable)]
pub fn derive_key(data: &[u8], salt: &[u8]) -> Result<SecVec<u8>, PWDuckCoreError> {
    let hasher = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::default(),
    );
    let mut password_hash: SecVec<u8> = vec![0_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into();
    hasher.hash_password_into(data, salt, &mut password_hash)?;
    Ok(password_hash)
}

/// Generate a new masterkey which will be encrypted with the given password after creation.
#[cfg_attr(test, mockable)]
pub fn generate_master_key(
    password: &str,
    key_file: Option<&Path>,
) -> Result<MasterKey, PWDuckCoreError> {
    // Generate random salt
    let salt = generate_salt();

    // Hash password with KDF or derive key from key file with KDF
    let hash = key_file.map_or_else(
        || hash_password(password, &salt),
        |path| {
            let key_file = generate_key_file(password, path)?;
            derive_key(&key_file, &salt)
        },
    )?;

    // Generate random initialization vector
    let iv = generate_aes_iv();

    // Generate random master key and encrypt it with password hash
    let mut master_key = [0_u8; MASTER_KEY_SIZE];
    //#[cfg(not(debug_assertions))]
    fill_random_bytes(&mut master_key);
    //#[cfg(debug_assertions)]
    //master_key
    //    .iter_mut()
    //    .enumerate()
    //    .for_each(|(i, x)| *x = (i % 16) as u8);

    let encrypted_key = aes_cbc_encrypt(&master_key, hash.as_slice(), &iv)?;
    master_key.zeroize();

    Ok(MasterKey::new(
        base64::encode(salt),
        base64::encode(iv),
        base64::encode(encrypted_key),
    ))
}

/// Decrypt a masterkey with the given password and encrypt it with the given memory key.
#[cfg_attr(test, mockable)]
pub fn decrypt_master_key(
    master_key: &MasterKey,
    password: &str,
    key_file: Option<&Path>,
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<crate::model::master_key::MasterKey, PWDuckCoreError> {
    let salt = base64::decode(master_key.salt())?;
    let mut hash = key_file.map_or_else(
        || hash_password(password, &salt),
        |path| {
            let key_file = crate::model::key_file::KeyFile::load(path, password)?;
            derive_key(&key_file, &salt)
        },
    )?;

    let encrypted_key = base64::decode(master_key.encrypted_key())?;
    let mut iv = base64::decode(master_key.iv())?;

    // unprotected key
    let mut key = aes_cbc_decrypt(&encrypted_key, &hash, &iv)?;
    hash.zeroize();
    iv.zeroize();

    // protect key
    let protected_key = protect_master_key(&key, key_protection, nonce)?;
    key.zeroize();

    Ok(protected_key.into())
}

/// Protect the masterkey by encrypting it with the given key.
#[cfg_attr(test, mockable)]
pub fn protect_master_key(
    master_key: &[u8],
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, PWDuckCoreError> {
    chacha20_encrypt(master_key, key_protection, nonce)
}

/// Unprotect the masterkey by decrypting it with the given key.
#[cfg_attr(test, mockable)]
pub fn unprotect_master_key(
    master_key: &[u8],
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<SecVec<u8>, PWDuckCoreError> {
    chacha20_decrypt(master_key, key_protection, nonce)
}

/// Generate a new key file as a 2nd factor authentification. It will be stored on the given path.
pub fn generate_key_file(
    password: &str,
    path: &Path,
) -> Result<crate::model::key_file::KeyFile, PWDuckCoreError> {
    // Genrate random salt
    let salt = generate_salt();

    // Hash password with KDF
    let password_hash = hash_password(password, &salt)?;

    // Generate random initialization vector
    let iv = generate_aes_iv();

    // Generate random key file and encrypt it with password hash
    let mut key_file: SecVec<u8> = vec![0_u8; KEY_FILE_SIZE].into();
    fill_random_bytes(&mut key_file);

    let encrypted_key = aes_cbc_encrypt(&key_file, password_hash.as_slice(), &iv)?;

    let dto = KeyFile::new(
        base64::encode(salt),
        base64::encode(iv),
        base64::encode(encrypted_key),
    );
    crate::io::save_key_file(path, dto)?;

    Ok(key_file.into())
}

/// Decrypt the key file with the given password.
pub fn decrypt_key_file(
    key_file: &KeyFile,
    password: &str,
) -> Result<crate::model::key_file::KeyFile, PWDuckCoreError> {
    let salt = base64::decode(key_file.salt())?;
    let mut hash = hash_password(password, &salt)?;

    let encrypted_key = base64::decode(key_file.encrypted_key())?;
    let mut iv = base64::decode(key_file.iv())?;

    let key = aes_cbc_decrypt(&encrypted_key, &hash, &iv)?;
    hash.zeroize();
    iv.zeroize();
    Ok(key.into())
}

/// Encrypt the data with the AES block cipher in CBC mode.
///
/// It expects:
///  - The data to encrypt
///  - The key for the encryption
///  - The iv for the CBC mode
#[cfg_attr(test, mockable)]
pub fn aes_cbc_encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, PWDuckCoreError> {
    let cipher = Cbc::<Aes256, Pkcs7>::new_from_slices(key, iv)?;
    Ok(cipher.encrypt_vec(data))
}

/// Decrypt the data with the AES block cipher in CBC mode.
///
/// It expects:
///  - The data to decrypt
///  - The key for the decryption
///  - The iv for the CBC mode
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
///  - The data to encrypt
///  - The key for the encryption
///  - A nonce (number-used-once)
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
///  - The data to decrypt
///  - The key for the decryption
///  - A nonce (number-used-once)
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
    use seckey::SecBytes;
    use tempfile::tempdir;

    use crate::mem_protection::MemKey;
    use crate::{PWDuckCoreError, SecVec};

    use super::{
        aes_cbc_decrypt, aes_cbc_encrypt, chacha20_decrypt, chacha20_encrypt, decrypt_key_file,
        decrypt_master_key, derive_key, derive_key_protection, fill_random_bytes, generate_aes_iv,
        generate_chacha20_nonce, generate_iv, generate_key_file, generate_master_key, generate_salt,
        hash_password, protect_master_key, unprotect_master_key, AES_IV_LENGTH,
        CHACHA20_NONCE_LENGTH, KEY_FILE_SIZE, MASTER_KEY_SIZE, SALT_LENGTH,
    };

    use mocktopus::mocking::*;

    const PASSWORD: &'static str = "This is a totally secret password";
    // const SALT: &'static str = "pa7lMD/slzor2CVNHZWNyA";
    const SALT: [u8; SALT_LENGTH] = [
        0xa5, 0xae, 0xe5, 0x30, 0x3f, 0xec, 0x97, 0x3a, 0x2b, 0xd8, 0x25, 0x4d, 0x1d, 0x95, 0x8d,
        0xc8,
    ];

    #[test]
    fn test_generate_iv() {
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(42);
            MockResult::Return(())
        });

        let iv = generate_iv(42);

        assert!(!iv.is_empty());
        assert_eq!(iv.len(), 42);
        assert_eq!(iv, vec![42_u8; 42]);
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

        let nonce = generate_chacha20_nonce().expect("Should not fail.");

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
        let salt1 = super::generate_salt();
        let salt2 = super::generate_salt();

        let hash1 = hash_password(PASSWORD, &salt1).expect("Hashing passwords should not fail");
        assert_ne!(hash1.as_slice(), PASSWORD.as_bytes());

        let hash2 = hash_password(PASSWORD, &salt2).expect("Hashing passwords should not fail");
        assert_ne!(hash1, hash2);

        let hash1_again =
            hash_password(PASSWORD, &salt1).expect("Hashing passwords should not fail");
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
                MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into()))
            } else {
                MockResult::Return(Err(PWDuckCoreError::Error(
                    "Not the expected MemKey".into(),
                )))
            }
        });

        let mem_key = MemKey::new();

        let key = derive_key_protection(&mem_key, &SALT).expect("Deriving key should not fail");
        let expected: SecVec<u8> = vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into();

        assert_eq!(key, expected);
    }

    #[test]
    fn test_derive_key() {
        let key = derive_key(PASSWORD.as_bytes(), &SALT).expect("Foo");

        let expected: SecVec<u8> = vec![
            72, 219, 9, 132, 177, 130, 185, 39, 90, 221, 173, 231, 171, 35, 7, 161, 205, 33, 148,
            192, 113, 22, 241, 202, 219, 231, 171, 134, 19, 56, 183, 152,
        ]
        .into();

        assert_eq!(key, expected);
    }

    #[test]
    fn test_generate_master_key_without_key() {
        let key1 =
            generate_master_key(PASSWORD, None).expect("Generating masterkey should not fail.");
        let key2 =
            generate_master_key(PASSWORD, None).expect("Generating masterkey should not fail.");

        assert_ne!(key1.salt(), key2.salt());
        assert_ne!(key1.iv(), key2.iv());
        assert_ne!(key1.encrypted_key(), key2.encrypted_key());

        let decrypted_key1 = aes_cbc_decrypt(
            &base64::decode(key1.encrypted_key()).unwrap(),
            &hash_password(PASSWORD, &base64::decode(key1.salt()).unwrap()).unwrap(),
            &base64::decode(key1.iv()).unwrap(),
        )
        .unwrap();

        let decrypted_key2 = aes_cbc_decrypt(
            &base64::decode(key2.encrypted_key()).unwrap(),
            &hash_password(PASSWORD, &base64::decode(key2.salt()).unwrap()).unwrap(),
            &base64::decode(key2.iv()).unwrap(),
        )
        .unwrap();

        assert_ne!(decrypted_key1, decrypted_key2);

        generate_salt.mock_safe(|| MockResult::Return(Vec::from(SALT)));
        hash_password.mock_safe(|pwd, salt| {
            assert_eq!(pwd, PASSWORD);
            assert_eq!(salt, &SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let master_key =
            generate_master_key(PASSWORD, None).expect("Generating masterkey should not fail.");

        let dercypted_key = aes_cbc_decrypt(
            &base64::decode(master_key.encrypted_key()).unwrap(),
            &hash_password(PASSWORD, &base64::decode(master_key.salt()).unwrap()).unwrap(),
            &base64::decode(master_key.iv()).unwrap(),
        )
        .unwrap();

        let expected: SecVec<u8> = vec![255_u8; MASTER_KEY_SIZE].into();

        assert_eq!(dercypted_key, expected);
    }

    #[test]
    fn test_generate_master_key_with_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("KeyFile.pwdk");

        assert!(!path.exists());

        let key = generate_master_key(PASSWORD, Some(&path))
            .expect("Generating master key should not fail.");

        assert!(path.exists());

        let key_file = crate::model::key_file::KeyFile::load(&path, PASSWORD).unwrap();

        let _ = aes_cbc_decrypt(
            &base64::decode(key.encrypted_key()).unwrap(),
            &derive_key(&key_file, &base64::decode(key.salt()).unwrap()).unwrap(),
            &base64::decode(key.iv()).unwrap(),
        )
        .expect("Decoding master key should not fail");
    }

    #[test]
    fn test_decrypt_master_key_without_key() {
        generate_salt.mock_safe(|| MockResult::Return(Vec::from(SALT)));
        hash_password.mock_safe(|password, salt| {
            assert_eq!(password, PASSWORD);
            assert_eq!(salt, SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let master_key = generate_master_key(PASSWORD, None).unwrap();

        MemKey::with_length.mock_safe(|length| {
            MockResult::Return(SecBytes::with(length, |buf| buf.fill(21_u8)).into())
        });

        let mem_key = MemKey::new();

        let salt = generate_salt();
        let nonce = generate_chacha20_nonce().unwrap();

        let key_protection = derive_key_protection(&mem_key, &salt).unwrap();
        unsafe {
            aes_cbc_decrypt.mock_raw(|encrypted_key, hash, iv| {
                assert_eq!(
                    encrypted_key,
                    base64::decode(master_key.encrypted_key()).unwrap()
                );
                assert_eq!(hash, &[42_u8; argon2::Params::DEFAULT_OUTPUT_LEN]);
                assert_eq!(iv, vec![21_u8; AES_IV_LENGTH]);
                MockResult::Continue((encrypted_key, hash, iv))
            });
        }

        unsafe {
            protect_master_key.mock_raw(|key, kp, no| {
                assert_eq!(
                    key,
                    aes_cbc_decrypt(
                        &base64::decode(master_key.encrypted_key()).unwrap(),
                        &hash_password(PASSWORD, &base64::decode(master_key.salt()).unwrap())
                            .unwrap(),
                        &base64::decode(master_key.iv()).unwrap(),
                    )
                    .unwrap()
                    .as_slice(),
                );
                assert_eq!(kp, key_protection.as_slice());
                assert_eq!(no, nonce);
                MockResult::Continue((key, kp, no))
            });
        }

        let decrypted_key = decrypt_master_key(&master_key, PASSWORD, None, &key_protection, &nonce)
            .expect("Decrypting masterkey should not fail.");

        let unprotected_key =
            unprotect_master_key(decrypted_key.as_slice(), &key_protection, &nonce)
                .expect("Unprotecting master_key should not fail");

        assert_eq!(
            unprotected_key,
            aes_cbc_decrypt(
                &base64::decode(master_key.encrypted_key()).unwrap(),
                &hash_password(PASSWORD, &base64::decode(master_key.salt()).unwrap()).unwrap(),
                &base64::decode(master_key.iv()).unwrap(),
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_decrypt_master_key_with_key() {
        generate_salt.mock_safe(|| MockResult::Return(Vec::from(SALT)));
        hash_password.mock_safe(|password, salt| {
            assert_eq!(password, PASSWORD);
            assert_eq!(salt, SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("KeyFile.pwdk");

        let master_key = generate_master_key(PASSWORD, Some(&path)).unwrap();

        MemKey::with_length.mock_safe(|length| {
            MockResult::Return(SecBytes::with(length, |buf| buf.fill(21_u8)).into())
        });

        let mem_key = MemKey::new();

        let salt = generate_salt();
        let nonce = generate_chacha20_nonce().unwrap();

        let key_protection = derive_key_protection(&mem_key, &salt).unwrap();

        let decrypted_key =
            decrypt_master_key(&master_key, PASSWORD, Some(&path), &key_protection, &nonce)
                .expect("Decrypting master_key should not fail.");

        let unprotected_key =
            unprotect_master_key(decrypted_key.as_slice(), &key_protection, &nonce)
                .expect("Unprotecting masterkey should not fail");

        let key_file = crate::model::key_file::KeyFile::load(&path, PASSWORD).unwrap();

        assert_eq!(
            unprotected_key,
            aes_cbc_decrypt(
                &base64::decode(master_key.encrypted_key()).unwrap(),
                &derive_key(&key_file, &base64::decode(master_key.salt()).unwrap()).unwrap(),
                &base64::decode(master_key.iv()).unwrap(),
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_protect_master_key() {
        let master_key = [255_u8; MASTER_KEY_SIZE];
        let key_protection = [42u8; argon2::Params::DEFAULT_OUTPUT_LEN];
        let nonce = [21_u8; CHACHA20_NONCE_LENGTH];

        unsafe {
            chacha20_encrypt.mock_raw(|m, k, n| {
                assert_eq!(m, master_key);
                assert_eq!(k, key_protection);
                assert_eq!(n, nonce);
                MockResult::Return(Ok(vec![84_u8; MASTER_KEY_SIZE]))
            });
        }

        let protected_key = protect_master_key(&master_key, &key_protection, &nonce)
            .expect("Protecting masterkey should not fail");
        assert_eq!(protected_key, vec![84_u8; MASTER_KEY_SIZE]);
    }

    #[test]
    fn test_unprotect_master_key() {
        let master_key = [84_u8; MASTER_KEY_SIZE];
        let key_protection = [42_u8; argon2::Params::DEFAULT_OUTPUT_LEN];
        let nonce = [21_u8; CHACHA20_NONCE_LENGTH];

        unsafe {
            chacha20_decrypt.mock_raw(|m, k, n| {
                assert_eq!(m, master_key);
                assert_eq!(k, key_protection);
                assert_eq!(n, nonce);
                MockResult::Return(Ok(vec![255_u8; MASTER_KEY_SIZE].into()))
            });
        }

        let unprotected_key = unprotect_master_key(&master_key, &key_protection, &nonce)
            .expect("Unprotecting masterkey should not fail");
        assert_eq!(unprotected_key, vec![255_u8; MASTER_KEY_SIZE].into())
    }

    #[test]
    fn test_generate_key_file() {
        generate_salt.mock_safe(|| MockResult::Return(Vec::from(SALT)));
        hash_password.mock_safe(|password, salt| {
            assert_eq!(password, PASSWORD);
            assert_eq!(salt, SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("KeyFile.pwdk");

        assert!(!path.exists());

        let key_file = generate_key_file(PASSWORD, &path).expect("Should not fail");

        assert!(path.exists());
        assert_eq!(
            key_file.key().as_slice(),
            vec![255u8; KEY_FILE_SIZE].as_slice()
        );

        let key_file_dto = crate::io::load_key_file(&path).unwrap();
        assert_eq!(
            base64::decode(key_file_dto.salt()).unwrap().as_slice(),
            &SALT
        );
        assert_eq!(
            base64::decode(key_file_dto.iv()).unwrap(),
            vec![21_u8; AES_IV_LENGTH]
        );
    }

    #[test]
    fn test_decrypt_key_file() {
        generate_salt.mock_safe(|| MockResult::Return(Vec::from(SALT)));
        hash_password.mock_safe(|password, salt| {
            assert_eq!(password, PASSWORD);
            assert_eq!(salt, SALT);
            MockResult::Return(Ok(vec![42_u8; argon2::Params::DEFAULT_OUTPUT_LEN].into()))
        });
        generate_iv.mock_safe(|len| MockResult::Return(vec![21_u8; len]));
        fill_random_bytes.mock_safe(|buf| {
            buf.fill(255_u8);
            MockResult::Return(())
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("KeyFile.pwdk");

        let key_file = generate_key_file(PASSWORD, &path).expect("Should not fail");

        let key_file_dto = crate::io::load_key_file(&path).unwrap();
        let decrypted_key_file =
            decrypt_key_file(&key_file_dto, PASSWORD).expect("Decrypting key file should not fail");

        assert_eq!(
            key_file.key().as_slice(),
            decrypted_key_file.key().as_slice()
        );
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
