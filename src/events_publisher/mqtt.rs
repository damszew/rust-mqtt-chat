use anyhow::Result;
use paho_mqtt::Message;
use tokio::sync::mpsc::UnboundedSender;

use crate::network::NetworkEvent;

use super::EventsPublisher;

pub struct MqttEventsPublisher {
    topic: String,
    sender: UnboundedSender<Message>,
}

impl MqttEventsPublisher {
    pub fn new(sender: UnboundedSender<Message>, topic: String) -> Self {
        Self { topic, sender }
    }
}

impl EventsPublisher for MqttEventsPublisher {
    type Message = NetworkEvent;

    fn publish(&self, message: Self::Message) -> Result<()> {
        match message {
            NetworkEvent::Message(msg) => {
                let msg = Message::new(&self.topic, msg, 0);
                self.sender.send(msg)?;
            }
        }
        Ok(())
    }
}
