use anyhow::Result;
use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder};
use tokio::sync::mpsc;

mod consumer;
mod publisher;

const CHANNEL_BUFFER: usize = 1;

#[derive(Debug, PartialEq)]
pub enum NetworkEvent {
    Message(Vec<u8>),
}

pub async fn network(
    url: impl Into<String>,
    topic_prefix: impl Into<String>,
    chat_room: impl Into<String>,
) -> Result<(mpsc::Sender<NetworkEvent>, mpsc::Receiver<NetworkEvent>)> {
    let topic_prefix = topic_prefix.into();
    let chat_room = chat_room.into();

    let opts = CreateOptionsBuilder::new().server_uri(url).finalize();
    let mut mqtt_client = AsyncClient::new(opts)?;

    let conn_opts = ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(30))
        .finalize();

    let mqtt_receiver = mqtt_client.get_stream(1);

    mqtt_client.connect(conn_opts).await?;

    let sub_topic = format!("{}/{}/#", topic_prefix, chat_room);
    mqtt_client.subscribe(&sub_topic, 0).wait()?;

    let topic = format!("{}/{}/user", topic_prefix, chat_room);

    let publisher_sender = {
        let (publisher_sender, publisher_receiver) = mpsc::channel(CHANNEL_BUFFER);
        let mut publisher = publisher::MqttPublisher {
            mqtt_client,
            topic,
            publishers: publisher_receiver,
        };
        tokio::spawn(async move { publisher.run().await });
        publisher_sender
    };

    let consumer_receiver = {
        let (consumer_sender, consumer_receiver) = mpsc::channel(CHANNEL_BUFFER);
        let mut consumer = consumer::MqttConsumer {
            mqtt_receiver,
            subscribers: vec![consumer_sender],
        };
        tokio::spawn(async move { consumer.run().await });
        consumer_receiver
    };

    Ok((publisher_sender, consumer_receiver))
}
