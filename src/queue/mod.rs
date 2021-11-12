type Message = Vec<u8>;
type Error = anyhow::Error;

#[async_trait::async_trait]
pub trait Queue {
    async fn publish(&self, topic: String, message: Message) -> Result<(), Error>;

    async fn subscribe(&mut self, topic: String) -> Result<(), Error>;

    async fn receive(&mut self) -> Result<Message, Error>;

    async fn run(&mut self) -> Result<(), Error>;
}
