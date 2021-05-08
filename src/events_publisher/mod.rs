use anyhow::Result;

pub mod mqtt;

pub trait EventsPublisher {
    type Message;

    fn publish(&self, message: Self::Message) -> Result<()>;
}
