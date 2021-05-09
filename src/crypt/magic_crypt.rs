use anyhow::Result;
use magic_crypt::{new_magic_crypt, MagicCrypt256, MagicCryptTrait};

use super::{Decrypt, Encrypt};

pub struct MagicEncrypt {
    crypt: MagicCrypt256,
}

impl MagicEncrypt {
    pub fn new(password: &impl AsRef<str>) -> Self {
        let crypt = new_magic_crypt!(password, 256);
        Self { crypt }
    }
}

impl Encrypt for MagicEncrypt {
    fn encrypt<T>(&self, data: T) -> Vec<u8>
    where
        T: AsRef<[u8]>,
    {
        self.crypt.encrypt_to_bytes(&data)
    }
}

pub struct MagicDecrypt {
    crypt: MagicCrypt256,
}

impl MagicDecrypt {
    pub fn new(password: &impl AsRef<str>) -> Self {
        let crypt = new_magic_crypt!(password, 256);
        Self { crypt }
    }
}

impl Decrypt for MagicDecrypt {
    fn decrypt<T>(&self, data: T) -> Result<Vec<u8>>
    where
        T: AsRef<[u8]>,
    {
        self.crypt
            .decrypt_bytes_to_bytes(&data)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
