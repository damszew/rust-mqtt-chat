use anyhow::Result;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait NetworkBackend {
    async fn recv(&mut self) -> Option<Vec<u8>>;
    async fn send(&mut self, msg: Vec<u8>) -> Result<()>;
}
