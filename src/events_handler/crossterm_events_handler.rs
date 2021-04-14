use crossterm::event::EventStream;
use crossterm::event::{Event, KeyCode, KeyEvent};
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

use super::EventHandler;
use crate::AppEvent;

pub struct CrosstermEventsHandler<S>
where
    S: Stream,
{
    sender: mpsc::Sender<AppEvent>,
    event_stream: S, // TODO: Try to make it `Fuse`
}

impl CrosstermEventsHandler<EventStream> {
    pub fn new(sender: mpsc::Sender<AppEvent>) -> Self {
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
                Event::Key(KeyEvent { code, modifiers: _ }) => match code {
                    KeyCode::Esc => {
                        self.sender.send(AppEvent::Quit).await?;
                        break;
                    }
                    KeyCode::Char(c) => {
                        self.sender.send(AppEvent::Character(c)).await?;
                    }
                    _ => {
                        todo!("Handle other KeyCodes")
                    }
                },
                Event::Mouse(_) => (),
                Event::Resize(_, _) => (),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyModifiers;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::AppEvent;

    #[tokio::test]
    async fn quit_on_esc() {
        let stream = tokio_stream::iter(vec![Ok(Event::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )))]);

        let (sender, mut receiver) = mpsc::channel(1);
        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            sender,
        };

        tested_event_handler.dispatch_events().await.unwrap();

        let event = tokio::time::timeout(std::time::Duration::from_millis(100), async move {
            receiver.recv().await.unwrap()
        })
        .await
        .unwrap();

        assert_eq!(event, AppEvent::Quit);
    }

    #[tokio::test]
    async fn shutdown_on_esc() {
        let stream = tokio_stream::iter(vec![
            Ok(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))),
            Ok(Event::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            ))),
        ]);

        let (sender, mut receiver) = mpsc::channel(1);
        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            sender,
        };

        let j = tokio::spawn(async move { tested_event_handler.dispatch_events().await });

        let event = tokio::time::timeout(std::time::Duration::from_millis(100), async move {
            receiver.recv().await.unwrap()
        })
        .await
        .unwrap();

        assert_eq!(event, AppEvent::Quit);

        let result = j.await.unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn char_on_input() {
        let stream = tokio_stream::iter(vec![Ok(Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        )))]);

        let (sender, mut receiver) = mpsc::channel(1);
        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            sender,
        };

        tested_event_handler.dispatch_events().await.unwrap();

        let event = tokio::time::timeout(std::time::Duration::from_millis(100), async move {
            receiver.recv().await.unwrap()
        })
        .await
        .unwrap();

        assert_eq!(event, AppEvent::Character('a'));
    }
}
