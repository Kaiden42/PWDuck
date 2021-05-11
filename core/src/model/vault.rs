//! TODO

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use zeroize::Zeroize;

use crate::{
    cryptography::{
        decrypt_masterkey, derive_key_protection, generate_argon2_salt, generate_chacha20_nonce,
        generate_masterkey, unprotect_masterkey,
    },
    error::PWDuckCoreError,
    io::{create_new_vault_dir, save_masterkey},
    mem_protection::MemKey,
};

use super::{entry::EntryHead, group::Group, master_key::MasterKey};

/// TODO
#[derive(Debug)]
pub struct Vault {
    masterkey: MasterKey,
    salt: String,
    nonce: Vec<u8>,
    path: PathBuf,
    groups: HashMap<String, Group>,
    entries: HashMap<String, EntryHead>,
}

impl Vault {
    /// TODO
    pub fn generate<P>(password: &str, mem_key: &MemKey, path: P) -> Result<Vault, PWDuckCoreError>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        create_new_vault_dir(&path)?;

        let masterkey_dto = generate_masterkey(password)?;

        //let salt = SaltString::generate(&mut OsRng);
        let salt = generate_argon2_salt();
        let nonce = generate_chacha20_nonce();

        let masterkey = decrypt_masterkey(
            &masterkey_dto,
            password,
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;
        /*let masterkey = MasterKey::load(
            &path,
            password,
            &derive_key_protection(mem_key, salt.as_str())?,
            &nonce
        )?;*/

        let mut vault = Vault {
            masterkey,
            salt,
            nonce,
            path,
            groups: HashMap::new(),
            entries: HashMap::new(),
        };

        let root = Group::root(vault.get_path());
        let _ = vault
            .get_groups_mut()
            .insert(root.get_uuid().as_string(), root);

        save_masterkey(vault.get_path(), masterkey_dto)?;
        vault.save(mem_key)?;

        Ok(vault)
    }

    /// TODO
    pub fn save(&mut self, mem_key: &MemKey) -> Result<(), PWDuckCoreError> {
        let path = self.path.to_owned();
        let mut masterkey = unprotect_masterkey(
            self.masterkey.get_key(),
            &derive_key_protection(mem_key, &self.salt)?,
            &self.nonce,
        )?;

        let group_result: Result<(), PWDuckCoreError> = self
            .groups
            .iter_mut()
            .filter(|(_, group)| group.is_modified())
            .map(|(_, group)| group.save(&path, &masterkey))
            .collect();

        let entry_result: Result<(), PWDuckCoreError> = self
            .entries
            .iter_mut()
            .filter(|(_, entry)| entry.is_modified())
            .map(|(_, entry)| entry.save(&path, &masterkey))
            .collect();

        masterkey.zeroize();

        group_result.and(entry_result)
    }

    /// TODO
    pub fn load<P>(password: &str, mem_key: &MemKey, path: P) -> Result<Vault, PWDuckCoreError>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        let salt = generate_argon2_salt();
        let nonce = generate_chacha20_nonce();

        //let masterkey = decrypt_masterkey(&masterkey_dto, password, &derive_key_protection(mem_key, salt.as_str())?, &nonce)?;
        let masterkey = MasterKey::load(
            &path,
            password,
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;

        let unprotected_masterkey = unprotect_masterkey(
            &masterkey.get_key(),
            &derive_key_protection(mem_key, &salt)?,
            &nonce,
        )?;
        let groups = Group::load_all(&path, &unprotected_masterkey)?;
        let entries = EntryHead::load_all(&path, &unprotected_masterkey)?;
        drop(unprotected_masterkey);

        let vault = Self {
            masterkey,
            salt,
            nonce,
            path,
            groups,
            entries,
        };

        Ok(vault)
    }

    /// TODO
    pub fn get_masterkey(&self) -> &MasterKey {
        &self.masterkey
    }

    /// TODO
    pub fn get_salt(&self) -> &str {
        &self.salt
    }

    /// TODO
    pub fn get_nonce(&self) -> &[u8] {
        &self.nonce
    }

    /// TODO
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// TODO
    pub fn get_groups(&self) -> &HashMap<String, Group> {
        &self.groups
    }

    /// TODO
    pub fn get_groups_mut(&mut self) -> &mut HashMap<String, Group> {
        &mut self.groups
    }

    /// TODO
    pub fn get_entries(&self) -> &HashMap<String, EntryHead> {
        &self.entries
    }
}

/// TODO
#[derive(Debug)]
pub struct Children {
    groups: Vec<String>,
    entries: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::mem_protection;

    use super::Vault;

    #[test]
    fn test_generate_new_vault() {
        let mem_key = mem_protection::MemKey::new();
        let path: PathBuf = "this_is_a_test_vault".into();

        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }

        let vault = Vault::generate("this is a pretty cool password", &mem_key, &path)
            .expect("Vault generation should not fail");

        // TODO
        std::fs::remove_dir_all(&path).unwrap();
    }
}
