use std::time::Duration;

use anyhow::Result;
use futures::{channel::mpsc::Receiver, StreamExt};
use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message};

use super::NetworkBackend;

pub struct MqttQueue {
    client: AsyncClient,
    mqtt_receiver: Receiver<Option<Message>>,
    topic: String,
}

impl MqttQueue {
    pub async fn new(
        url: impl Into<String>,
        topic_prefix: impl Into<String>,
        chat_room: impl Into<String>,
    ) -> Result<Self> {
        let topic_prefix = topic_prefix.into();
        let chat_room = chat_room.into();

        let opts = CreateOptionsBuilder::new().server_uri(url).finalize();
        let mut client = AsyncClient::new(opts)?;

        let conn_opts = ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(30))
            .finalize();

        let mqtt_receiver = client.get_stream(1);

        client.connect(conn_opts).await?;

        let sub_topic = format!("{}/{}/#", topic_prefix, chat_room);
        client.subscribe(&sub_topic, 0).wait()?;

        let topic = format!("{}/{}/user", topic_prefix, chat_room);
        Ok(Self {
            client,
            mqtt_receiver,
            topic,
        })
    }
}

#[async_trait::async_trait]
impl NetworkBackend for MqttQueue {
    async fn recv(&mut self) -> Option<Vec<u8>> {
        if let Some(Some(msg)) = self.mqtt_receiver.next().await {
            Some(msg.payload().to_owned())
        } else {
            None
        }
    }

    async fn send(&mut self, msg: Vec<u8>) -> Result<()> {
        let mqtt_message = Message::new(&self.topic, msg, 0);
        self.client.publish(mqtt_message).await?;
        Ok(())
    }
}
