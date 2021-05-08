use actor_model_chat::{
    app::App,
    events_publisher::mqtt::MqttEventsPublisher,
    events_reader::{mqtt::MqttEventsReader, terminal::CrosstermEventsHandler},
    renderer::terminal_renderer::TerminalRenderer,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let (events_publisher, network_events) =
        setup_network("tcp://localhost:1883/", "topic", "chat").await?;
    let terminal_events = CrosstermEventsHandler::new();
    let renderer = TerminalRenderer::new(std::io::stdout())?;

    let mut app = App::new(network_events, terminal_events, renderer, events_publisher).await;

    app.run().await?;

    Ok(())
}

pub async fn setup_network(
    url: impl Into<String>,
    topic_prefix: impl Into<String>,
    chat_room: impl Into<String>,
) -> Result<(MqttEventsPublisher, MqttEventsReader)> {
    // TODO: clean up
    let topic_prefix = topic_prefix.into();
    let chat_room = chat_room.into();

    let opts = paho_mqtt::CreateOptionsBuilder::new()
        .server_uri(url)
        .finalize();
    let mut mqtt_client = paho_mqtt::AsyncClient::new(opts)?;

    let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(std::time::Duration::from_secs(30))
        .finalize();

    let recv = mqtt_client.get_stream(1);
    let events_reader = MqttEventsReader::new(recv);

    mqtt_client.connect(conn_opts).await?;

    let sub_topic = format!("{}/{}/#", topic_prefix, chat_room);
    mqtt_client.subscribe(&sub_topic, 0).wait()?;

    let topic = format!("{}/{}/user", topic_prefix, chat_room);

    let (sender, mut recv) = tokio::sync::mpsc::unbounded_channel();
    let events_publisher = MqttEventsPublisher::new(sender, topic);

    tokio::spawn(async move {
        while let Some(message) = recv.recv().await {
            mqtt_client.publish(message).await.unwrap();
        }
    });

    Ok((events_publisher, events_reader))
}
