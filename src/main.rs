use std::thread::sleep;

use actor_model_chat::{
    events_handler::{crossterm_events_handler::CrosstermEventsHandler, EventHandler},
    renderer::{terminal_renderer::TerminalRenderer, Renderer, State},
};
use anyhow::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let mut renderer = TerminalRenderer::new(std::io::stdout())?;

    let (s, r) = mpsc::channel(1);

    let mut events_handler = CrosstermEventsHandler::new(s);

    events_handler.dispatch_events().await?;

    let state = State::default();

    renderer.render(&state)?;

    sleep(std::time::Duration::from_secs(2));

    Ok(())
}
