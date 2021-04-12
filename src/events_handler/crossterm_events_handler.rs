use crossterm::event::EventStream;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use futures::{stream::Fuse, Stream, StreamExt};
use tokio::sync::mpsc;

use super::EventHandler;

pub struct CrosstermEventsHandler {
    sender: mpsc::Sender<()>,
    event_stream: Fuse<EventStream>, // do not allow calls to stream after it finished
}

impl CrosstermEventsHandler {
    pub fn new(sender: mpsc::Sender<()>) -> Self {
        let event_stream = EventStream::new().fuse();

        Self {
            sender,
            event_stream,
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for CrosstermEventsHandler {
    async fn dispatch_events(&mut self) -> anyhow::Result<()> {
        while let Some(Ok(event)) = self.event_stream.next().await {
            match event {
                Event::Key(key_event) => {
                    dbg!(key_event);
                }
                Event::Mouse(_) => (),
                Event::Resize(_, _) => (),
            }
        }

        Ok(())
    }
}

// TODO: write FakeEventStream for test
