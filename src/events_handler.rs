pub mod crossterm_events_handler;

use anyhow::Result;

#[async_trait::async_trait]
pub trait EventHandler {
    // TODO: get rid of it as it server no purpose
    async fn dispatch_events(&mut self) -> Result<()>;
}
