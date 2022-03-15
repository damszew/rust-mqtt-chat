use anyhow::Result;

pub mod magic_crypt;

#[cfg_attr(test, mockall::automock)]
pub trait Encrypt {
    /// Returns base64 encrypted string
    fn encrypt<T>(&self, data: T) -> Vec<u8>
    where
        T: AsRef<[u8]> + 'static; // 'static needed only for mocking purposes
}

#[cfg_attr(test, mockall::automock)]

pub trait Decrypt {
    /// Expects input as a base64 string
    fn decrypt<T>(&self, data: T) -> Result<Vec<u8>>
    where
        T: AsRef<[u8]> + 'static; // 'static needed only for mocking purposes
}

#[cfg(test)]
mockall::mock! {

    pub Crypto {}

    impl Encrypt for Crypto {
        fn encrypt<T>(&self, data: T) -> Vec<u8>
        where
            T: AsRef<[u8]> + 'static;
    }

    impl Decrypt for Crypto {
        fn decrypt<T>(&self, data: T) -> Result<Vec<u8>>
        where
            T: AsRef<[u8]> + 'static;
    }

    impl Clone for Crypto {
        fn clone(&self) -> Self;
    }
}
