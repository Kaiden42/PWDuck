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

/// The length of the initialization vector for the AES encryption.
const AES_IV_LENGTH: usize = 16;
/// The length of the nonce for the `ChaCha20` encryption.
const CHACHA20_NONCE_LENGTH: usize = 12;

/// Generate a new random initialization vector for the AES encryption.
pub fn generate_aes_iv() -> Vec<u8> {
    generate_iv(AES_IV_LENGTH)
}

/// Generate a new random nonce (number-used-once) for the `ChaCha20` encryption.
pub fn generate_chacha20_nonce() -> Vec<u8> {
    generate_iv(CHACHA20_NONCE_LENGTH)
}

/// Generate a new random salt for the Argon2 key derivation.
pub fn generate_argon2_salt() -> String {
    // TODO
    SaltString::generate(&mut OsRng).as_str().to_owned()
}

/// Generate a new random iv with the given length.
fn generate_iv(length: usize) -> Vec<u8> {
    let mut iv: Vec<u8> = vec![0_u8; length];
    //OsRng.fill_bytes(&mut iv);
    fill_random_bytes(&mut iv);
    iv
}

/// Fill the given slice of bytes with random values.
pub fn fill_random_bytes(buf: &mut [u8]) {
    //let mut iv = vec![0u8; length];
    let mut rng = ChaCha20Rng::from_entropy();
    rng.fill_bytes(buf);
}

/// Hash the password.
pub fn hash_password(password: &str, salt: &str) -> Result<SecVec<u8>, PWDuckCoreError> {
    derive_key(password.as_bytes(), salt)
}

/// Derive a memory key.
pub fn derive_key_protection(mem_key: &MemKey, salt: &str) -> Result<SecVec<u8>, PWDuckCoreError> {
    derive_key(&mem_key.read(), salt)
}

/// Derive a key from date based on the given salt.
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
pub fn generate_masterkey(password: &str) -> Result<MasterKey, PWDuckCoreError> {
    // Generate random salt
    let salt = SaltString::generate(&mut OsRng);

    // Hash password with KDF
    let password_hash = hash_password(password, salt.as_str())?;

    // Generate random initialization vector
    //let mut iv = [0u8; 16];
    //OsRng.fill_bytes(&mut iv);
    let iv = generate_aes_iv();

    // Generate random master key and encrypt it with password hash
    let mut master_key = [0_u8; 32];
    OsRng.fill_bytes(&mut master_key);
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
pub fn protect_masterkey(
    master_key: &[u8],
    key_protection: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, PWDuckCoreError> {
    chacha20_encrypt(master_key, key_protection, nonce)
}

/// Unprotect the masterkey by decrypting it with the given key.
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

    use crate::{cryptography::chacha20_decrypt, mem_protection::MemKey};

    use super::generate_masterkey;
    use super::hash_password;
    use super::{
        aes_cbc_decrypt, aes_cbc_encrypt, chacha20_encrypt, decrypt_masterkey,
        derive_key_protection, fill_random_bytes, generate_aes_iv, generate_argon2_salt,
        generate_chacha20_nonce,
    };
    use super::{AES_IV_LENGTH, CHACHA20_NONCE_LENGTH};

    // TODO: mocking

    #[test]
    fn test_generate_aes_iv() {
        let iv1 = generate_aes_iv();

        assert!(!iv1.is_empty());
        assert_eq!(iv1.len(), AES_IV_LENGTH);

        let iv2 = generate_aes_iv();

        assert_ne!(iv1, iv2);
    }

    #[test]
    fn test_generate_chacha20_nonce() {
        let nonce1 = generate_chacha20_nonce();

        assert!(!nonce1.is_empty());
        assert_eq!(nonce1.len(), CHACHA20_NONCE_LENGTH);

        let nonce2 = generate_chacha20_nonce();

        assert_ne!(nonce1, nonce2);
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

        let password = "this is a secret password";

        let hash1 =
            hash_password(password, salt1.as_str()).expect("Hashing passwords should not fail");
        assert_ne!(hash1.as_slice(), password.as_bytes());

        let hash2 =
            hash_password(password, salt2.as_str()).expect("Hashing passwords should not fail");
        assert_ne!(hash1, hash2);

        let hash1_again =
            hash_password(password, salt1.as_str()).expect("Hashing passwords should not fail");
        assert_eq!(hash1, hash1_again);
    }

    #[test]
    fn test_derive_key_protection() {
        // TODO
    }

    #[test]
    fn test_derive_key() {
        // TODO
    }

    #[test]
    fn test_generate_masterkey() {
        let password = "totally secret password";

        let key1 = generate_masterkey(password).expect("Generating masterkey should not fail");
        let key2 = generate_masterkey(password).expect("Generating masterkey should not fail");

        assert_ne!(key1.salt(), key2.salt());
        assert_ne!(key1.iv(), key2.iv());
        assert_ne!(key1.encrypted_key(), key2.encrypted_key());
    }

    #[test]
    fn test_decrypt_masterkey() {
        let password = "totally secret password";
        let masterkey = generate_masterkey(password).expect("Generating masterkey should not fail");

        let mem_key = MemKey::new();
        let salt = generate_argon2_salt();
        let nonce = generate_chacha20_nonce();

        let key_protection = derive_key_protection(&mem_key, salt.as_str())
            .expect("Deriving key protection should not fail");

        let _decrypted_key = decrypt_masterkey(&masterkey, password, &key_protection, &nonce)
            .expect("Decrypting master key should not fail");

        // TODO check if this has worked by mocking or something
    }

    #[test]
    fn test_protect_masterkey() {
        // TODO
    }

    #[test]
    fn test_unprotect_masterkey() {
        // TODO
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
