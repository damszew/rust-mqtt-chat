use anyhow::Result;

pub mod mqtt;
pub mod terminal;

#[async_trait::async_trait]
pub trait EventsReader {
    type Message;

    async fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(Self::Message) + Send + 'static;

    async fn run(&mut self) -> Result<()>;
}
