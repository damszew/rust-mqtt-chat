use actor_model_chat::{
    app::App,
    events_handler::{crossterm_events_handler::CrosstermEventsHandler, EventHandler},
    network::network,
    renderer::terminal_renderer::TerminalRenderer,
};
use anyhow::Result;
use tokio::try_join;
use tokio::{sync::mpsc, task};

#[tokio::main]
async fn main() -> Result<()> {
    let (publisher, consumer) = network("http://localhost:1883/", "topic", "chat").await?;

    let renderer = TerminalRenderer::new(std::io::stdout())?;

    let (sender, receiver) = mpsc::channel(1);
    let mut events_handler = CrosstermEventsHandler::new(sender);
    let mut app = App::new(receiver, renderer);

    let event_task = task::spawn(async move { events_handler.dispatch_events().await });
    let app_task = task::spawn(async move { app.run().await });

    let results = try_join!(event_task, app_task)?;
    results.0?;
    results.1?;

    Ok(())
}
