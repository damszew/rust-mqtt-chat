use actor_model_chat::{
    app::App, events_reader::terminal::CrosstermEventsHandler, network::setup_network,
    renderer::terminal_renderer::TerminalRenderer,
};
use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (events_publisher, network_events) =
        setup_network("tcp://localhost:1883/", "topic", "chat").await?;
    let terminal_events = CrosstermEventsHandler::new();
    let renderer = TerminalRenderer::new(std::io::stdout())?;

    let mut app = App::new(network_events, terminal_events, renderer, events_publisher).await;

    app.run().await?;

    Ok(())
}
