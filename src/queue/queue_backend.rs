use anyhow::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct BackendMessage {
    pub topic: String,
    pub payload: Vec<u8>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait QueueBackend {
    async fn connect(&mut self) -> Result<()>;
    async fn subscribe(&mut self, topic: &str) -> Result<()>;
    async fn recv(&mut self) -> Option<BackendMessage>;
    async fn send(&mut self, msg: BackendMessage) -> Result<()>;
}
