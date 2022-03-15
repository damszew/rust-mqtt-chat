use std::pin::Pin;

use futures::Stream;

type Message = Vec<u8>;
type Error = anyhow::Error;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Queue {
    async fn publish(&self, topic: String, message: Message) -> Result<(), Error>;

    async fn subscribe(&mut self, topic: String) -> Result<(), Error>;

    // Dynamic dispatch because of error[E0562]: `impl Trait` not allowed outside of function and method return types
    fn stream(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message, Error>>>>;
}

pub mod encrypted_queue;
pub mod mqtt;
