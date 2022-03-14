use anyhow::Result;
use magic_crypt::{new_magic_crypt, MagicCrypt256, MagicCryptTrait};

use super::{Decrypt, Encrypt};

#[derive(Clone)]
pub struct MagicCrypt {
    crypt: MagicCrypt256,
}

impl MagicCrypt {
    pub fn new(password: &impl AsRef<str>) -> Self {
        let crypt = new_magic_crypt!(password, 256);
        Self { crypt }
    }
}

impl Encrypt for MagicCrypt {
    fn encrypt<T>(&self, data: T) -> Vec<u8>
    where
        T: AsRef<[u8]>,
    {
        self.crypt.encrypt_to_bytes(&data)
    }
}

impl Decrypt for MagicCrypt {
    fn decrypt<T>(&self, data: T) -> Result<Vec<u8>>
    where
        T: AsRef<[u8]>,
    {
        self.crypt
            .decrypt_bytes_to_bytes(&data)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
