use std::{pin::Pin, sync::Arc};

use futures::{channel::mpsc, lock::Mutex, Stream, StreamExt};

use super::{Error, Message, Queue};

#[derive(Clone)]
pub struct MqttQueue {
    client: paho_mqtt::AsyncClient,
    receiver: Arc<Mutex<mpsc::Receiver<Option<paho_mqtt::Message>>>>,
}

impl MqttQueue {
    pub async fn new(url: String) -> Result<Self, Error> {
        let opts = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(url)
            .finalize();
        let mut client = paho_mqtt::AsyncClient::new(opts)?;
        let receiver = Arc::new(Mutex::new(client.get_stream(1)));

        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(std::time::Duration::from_secs(30))
            .finalize();
        client.connect(conn_opts).await?;

        Ok(Self { client, receiver })
    }
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

    fn stream(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message, Error>>>> {
        let stream = self
            .client
            .get_stream(1)
            .map(|m| m.map(|msg| msg.payload().to_owned()))
            .map(|option| option.ok_or_else(|| anyhow::anyhow!("Disconnected")));

        stream.boxed()
    }
}
