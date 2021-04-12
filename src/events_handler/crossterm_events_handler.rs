use crossterm::event::EventStream;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use futures::{stream::Fuse, Stream, StreamExt};
use tokio::sync::mpsc;

use super::EventHandler;

pub struct CrosstermEventsHandler<S>
where
    S: Stream,
{
    sender: mpsc::Sender<()>,
    event_stream: S, // TODO: Try to make it `Fuse`
}

impl CrosstermEventsHandler<EventStream> {
    pub fn new(sender: mpsc::Sender<()>) -> Self {
        let event_stream = EventStream::new();

        Self {
            sender,
            event_stream,
        }
    }
}

#[async_trait::async_trait]
impl<S> EventHandler for CrosstermEventsHandler<S>
where
    S: Stream<Item = crossterm::Result<Event>> + Unpin + Send,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_name() {
        let event = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        let (sender, mut receiver) = mpsc::channel(1);

        let stream = tokio_stream::iter(vec![Ok(event)]);

        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            sender,
        };

        tested_event_handler.dispatch_events().await.unwrap();

        let event = tokio::time::timeout(std::time::Duration::from_millis(100), async move {
            receiver.recv().await.unwrap();
        })
        .await
        .unwrap();
    }
}
