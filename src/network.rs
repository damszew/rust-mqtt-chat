use anyhow::Result;
use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder};
use tokio::sync::mpsc;

use crate::{
    crypto::magic_crypt::{MagicDecrypt, MagicEncrypt},
    events_publisher::{mqtt::MqttEventsPublisher, EventsPublisher},
    events_reader::{mqtt::MqttEventsReader, EventsReader},
};

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEvent {
    Message(Vec<u8>),
}

pub async fn setup_network(
    url: impl Into<String>,
    topic_prefix: impl Into<String>,
    chat_room: impl Into<String>,
    user: impl Into<String>,
    password: &impl AsRef<str>,
) -> Result<(
    impl EventsPublisher<Message = NetworkEvent>,
    impl EventsReader<Message = NetworkEvent>,
)> {
    // TODO: clean up
    let topic_prefix = topic_prefix.into();
    let chat_room = chat_room.into();
    let user = user.into();

    let opts = CreateOptionsBuilder::new().server_uri(url).finalize();
    let mut mqtt_client = AsyncClient::new(opts)?;

    let conn_opts = ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(30))
        .finalize();

    let recv = mqtt_client.get_stream(1);
    let decryptor = MagicDecrypt::new(password);
    let events_reader = MqttEventsReader::new(recv, decryptor);

    mqtt_client.connect(conn_opts).await?;

    let sub_topic = format!("{}/{}/#", topic_prefix, chat_room);
    mqtt_client.subscribe(&sub_topic, 0).wait()?;

    let topic = format!("{}/{}/{}", topic_prefix, chat_room, user);

    let (sender, mut recv) = mpsc::unbounded_channel();
    let encryptor = MagicEncrypt::new(password);
    let events_publisher = MqttEventsPublisher::new(sender, topic, encryptor);

    tokio::spawn(async move {
        while let Some(message) = recv.recv().await {
            mqtt_client.publish(message).await.unwrap();
        }
    });

    Ok((events_publisher, events_reader))
}
