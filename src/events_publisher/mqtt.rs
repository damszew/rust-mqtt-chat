use anyhow::Result;
use paho_mqtt::Message;
use tokio::sync::mpsc::UnboundedSender;

use crate::{crypt::Encrypt, network::NetworkEvent};

use super::EventsPublisher;

pub struct MqttEventsPublisher<E>
where
    E: Encrypt,
{
    topic: String,
    encryptor: E,
    sender: UnboundedSender<Message>,
}

impl<E> MqttEventsPublisher<E>
where
    E: Encrypt,
{
    pub fn new(sender: UnboundedSender<Message>, topic: String, encryptor: E) -> Self {
        Self {
            topic,
            encryptor,
            sender,
        }
    }
}

impl<E> EventsPublisher for MqttEventsPublisher<E>
where
    E: Encrypt,
{
    type Message = NetworkEvent;

    fn publish(&self, message: Self::Message) -> Result<()> {
        match message {
            NetworkEvent::Message(msg) => {
                let msg = self.encryptor.encrypt(msg);
                let msg = Message::new(&self.topic, msg, 0);
                self.sender.send(msg)?;
            }
        }
        Ok(())
    }
}
