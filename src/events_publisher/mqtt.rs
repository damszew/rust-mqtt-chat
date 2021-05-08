use anyhow::Result;
use paho_mqtt::{AsyncClient, Message};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::network::NetworkEvent;

use super::EventsPublisher;

pub struct MqttEventsPublisher {
    mqtt_client: AsyncClient,
    topic: String,
    recv: UnboundedReceiver<NetworkEvent>,
    sender: UnboundedSender<NetworkEvent>,
}

impl MqttEventsPublisher {
    pub fn new(mqtt_client: AsyncClient, topic: String) -> Self {
        let (sender, recv) = unbounded_channel();

        Self {
            mqtt_client,
            topic,
            recv,
            sender,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.recv.recv().await {
            match message {
                NetworkEvent::Message(msg) => {
                    let msg = Message::new(&self.topic, msg, 0);
                    self.mqtt_client.publish(msg).await?;
                }
            }
        }

        Ok(())
    }
}

impl EventsPublisher for MqttEventsPublisher {
    type Message = NetworkEvent;

    fn publish(&self, message: Self::Message) -> Result<()> {
        self.sender.send(message)?;
        Ok(())
    }
}
