use anyhow::Result;

pub trait EventsPublisher {
    type Message;

    fn publish(&self, message: Self::Message) -> Result<()>;
}
