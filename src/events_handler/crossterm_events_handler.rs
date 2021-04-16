use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
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
        while let Some(event) = self.event_stream.next().await {
            let event = event?;
            if let Event::Key(KeyEvent { code, modifiers }) = event {
                match code {
                    KeyCode::Esc => {
                        self.sender.send(AppEvent::Quit).await?;
                        break;
                    }
                    KeyCode::Char(c) => {
                        if c == 'c' && modifiers == KeyModifiers::CONTROL {
                            self.sender.send(AppEvent::Quit).await?;
                            break;
                        } else {
                            self.sender.send(AppEvent::Character(c)).await?;
                        }
                    }
                    KeyCode::Enter => {
                        self.sender.send(AppEvent::Accept).await?;
                    }
                    KeyCode::Delete => {
                        self.sender.send(AppEvent::Remove).await?;
                    }
                    KeyCode::Backspace => {
                        self.sender.send(AppEvent::RemoveLast).await?;
                    }
                    KeyCode::Left => {
                        self.sender.send(AppEvent::CursorLeft).await?;
                    }
                    KeyCode::Right => {
                        self.sender.send(AppEvent::CursorRight).await?;
                    }
                    KeyCode::Home => {
                        self.sender.send(AppEvent::CursorStart).await?;
                    }
                    KeyCode::End => {
                        self.sender.send(AppEvent::CursorEnd).await?;
                    }

                    KeyCode::Up => {
                        self.sender.send(AppEvent::ScrollUp).await?;
                    }
                    KeyCode::Down => {
                        self.sender.send(AppEvent::ScrollDown).await?;
                    }
                    _ => {}
                }
            } // Ignore other events
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};
    use test_case::test_case;

    use super::*;
    use crate::AppEvent;

    #[test_case( KeyCode::Esc, KeyModifiers::NONE ; "on esc")]
    #[test_case( KeyCode::Char('c'), KeyModifiers::CONTROL ; "on ctrl c")]
    #[tokio::test]
    async fn quit_and_shutdown(exit_key: KeyCode, modifier: KeyModifiers) {
        let stream = tokio_stream::iter(vec![
            Ok(Event::Key(KeyEvent::new(exit_key, modifier))),
            Err(crossterm::ErrorKind::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "oh no!",
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

    #[test_case( 'a', KeyModifiers::NONE ; "lowercase")]
    #[test_case( 'D', KeyModifiers::NONE ; "uppercase")]
    #[test_case( 't', KeyModifiers::CONTROL ; "ignore ctrl in other than ctrl c")]
    #[test_case( 'o', KeyModifiers::ALT ; "ignore alt")]
    #[tokio::test]
    async fn letters_without_modifiers(c: char, modifier: KeyModifiers) {
        let stream = tokio_stream::iter(vec![Ok(Event::Key(KeyEvent::new(
            KeyCode::Char(c),
            modifier,
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

        assert_eq!(event, AppEvent::Character(c));
    }

    #[test_case( KeyCode::Enter => AppEvent::Accept ; "enter")]
    #[test_case( KeyCode::Delete => AppEvent::Remove ; "delete")]
    #[test_case( KeyCode::Backspace => AppEvent::RemoveLast ; "backspace")]
    #[test_case( KeyCode::Left => AppEvent::CursorLeft ; "left")]
    #[test_case( KeyCode::Right => AppEvent::CursorRight ; "right")]
    #[test_case( KeyCode::Home => AppEvent::CursorStart ; "home")]
    #[test_case( KeyCode::End => AppEvent::CursorEnd ; "end")]
    #[test_case( KeyCode::Up => AppEvent::ScrollUp ; "up")]
    #[test_case( KeyCode::Down => AppEvent::ScrollDown ; "down")]
    #[tokio::test]
    async fn special_key(key: KeyCode) -> AppEvent {
        let stream =
            tokio_stream::iter(vec![Ok(Event::Key(KeyEvent::new(key, KeyModifiers::NONE)))]);

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

        event
    }

    #[test_case( Event::Mouse(
                    MouseEvent {
                        kind: MouseEventKind::Moved,
                        column: 0,
                        row: 0,
                        modifiers: KeyModifiers::NONE,
                    })
        ; "mouse")]
    #[test_case( Event::Resize(0, 0) ; "resize")]
    #[tokio::test]
    async fn ignore(event: Event) {
        let stream = tokio_stream::iter(vec![Ok(event)]);

        let (sender, mut receiver) = mpsc::channel(1);
        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            sender,
        };

        tested_event_handler.dispatch_events().await.unwrap();

        let result = tokio::time::timeout(std::time::Duration::from_millis(100), async move {
            receiver.recv().await.unwrap()
        })
        .await;

        assert!(result.is_err()) // We ignore events so timeouts are triggered
    }
}
