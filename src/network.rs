use anyhow::Result;
use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder};
use tokio::sync::mpsc;

const CHANNEL_BUFFER: usize = 1;

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEvent {
    Message(Vec<u8>),
}

pub async fn network(
    url: impl Into<String>,
    topic_prefix: impl Into<String>,
    chat_room: impl Into<String>,
) -> Result<(mpsc::Sender<NetworkEvent>, mpsc::Receiver<NetworkEvent>)> {
    // TODO: clean up
    let topic_prefix = topic_prefix.into();
    let chat_room = chat_room.into();

    let opts = CreateOptionsBuilder::new().server_uri(url).finalize();
    let mut mqtt_client = AsyncClient::new(opts)?;

    let conn_opts = ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(30))
        .finalize();

    let _ = mqtt_client.get_stream(1);

    mqtt_client.connect(conn_opts).await?;

    let sub_topic = format!("{}/{}/#", topic_prefix, chat_room);
    mqtt_client.subscribe(&sub_topic, 0).wait()?;

    let _ = format!("{}/{}/user", topic_prefix, chat_room);

    let publisher_sender = {
        let (publisher_sender, _) = mpsc::channel(CHANNEL_BUFFER);
        publisher_sender
    };

    let consumer_receiver = {
        let (_, consumer_receiver) = mpsc::channel(CHANNEL_BUFFER);
        consumer_receiver
    };

    Ok((publisher_sender, consumer_receiver))
}
