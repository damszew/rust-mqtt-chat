use anyhow::Result;
use paho_mqtt::{AsyncClient, Message};
use tokio::sync::mpsc;

use super::NetworkEvent;

pub struct MqttPublisher {
    pub mqtt_client: AsyncClient,
    pub topic: String,
    pub publishers: mpsc::Receiver<NetworkEvent>,
}

impl MqttPublisher {
    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.publishers.recv().await {
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
