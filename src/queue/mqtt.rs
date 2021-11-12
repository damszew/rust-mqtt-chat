use futures::StreamExt;

use super::{Error, Message, Queue};

pub struct MqttQueue {
    client: paho_mqtt::AsyncClient,
    receiver: futures::channel::mpsc::Receiver<Option<paho_mqtt::Message>>,
}

#[async_trait::async_trait]
impl Queue for MqttQueue {
    async fn publish(&self, topic: String, message: Message) -> Result<(), Error> {
        let mqtt_msg = paho_mqtt::Message::new(topic, message, 0);
        self.client.publish(mqtt_msg).await?;

        Ok(())
    }

    async fn subscribe(&mut self, topic: String) -> Result<(), Error> {
        self.client.subscribe(&topic, 0).await?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Message, Error> {
        let msg = self
            .receiver
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("Error"))?
            .ok_or_else(|| anyhow::anyhow!("Error"))?;

        Ok(msg.payload().to_owned())
    }
}
