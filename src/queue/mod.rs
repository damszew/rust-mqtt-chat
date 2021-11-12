type Message = Vec<u8>;
type Error = anyhow::Error;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Queue {
    async fn publish(&self, topic: String, message: Message) -> Result<(), Error>;

    async fn subscribe(&mut self, topic: String) -> Result<(), Error>;

    async fn receive(&mut self) -> Result<Message, Error>;
}

pub mod encrypted_queue;
pub mod mqtt;
