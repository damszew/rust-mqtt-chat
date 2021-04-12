pub mod crossterm_events_handler;

use anyhow::Result;

#[async_trait::async_trait]
pub trait EventHandler {
    async fn dispatch_events(&mut self) -> Result<()>;
}
