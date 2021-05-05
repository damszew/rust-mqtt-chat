use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::{Stream, StreamExt};

use super::EventsReader;
use crate::AppEvent;

pub struct CrosstermEventsHandler<S>
where
    S: Stream,
{
    subscribers: Vec<Box<dyn Fn(AppEvent) -> () + Send + 'static>>,
    event_stream: S, // TODO: Try to make it `Fuse`
}

impl CrosstermEventsHandler<EventStream> {
    pub fn new() -> Self {
        let event_stream = EventStream::new();

        Self {
            subscribers: Vec::new(),
            event_stream,
        }
    }
}

#[async_trait::async_trait]
impl<S> EventsReader for CrosstermEventsHandler<S>
where
    S: Stream<Item = crossterm::Result<Event>> + Unpin + Send,
{
    type Message = AppEvent;

    async fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(Self::Message) -> () + Send + 'static,
    {
        self.subscribers.push(Box::new(callback));
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.event_stream.next().await {
            let event = event?;
            if let Event::Key(KeyEvent { code, modifiers }) = event {
                match code {
                    KeyCode::Esc => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::Quit));
                        break;
                    }
                    KeyCode::Char(c) => {
                        if c == 'c' && modifiers == KeyModifiers::CONTROL {
                            self.subscribers
                                .iter()
                                .for_each(|subscriber| subscriber(AppEvent::Quit));
                            break;
                        } else {
                            self.subscribers
                                .iter()
                                .for_each(|subscriber| subscriber(AppEvent::Character(c)));
                        }
                    }
                    KeyCode::Enter => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::Accept));
                    }
                    KeyCode::Delete => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::Remove));
                    }
                    KeyCode::Backspace => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::RemoveLast));
                    }
                    KeyCode::Left => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::CursorLeft));
                    }
                    KeyCode::Right => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::CursorRight));
                    }
                    KeyCode::Home => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::CursorStart));
                    }
                    KeyCode::End => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::CursorEnd));
                    }

                    KeyCode::Up => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::ScrollUp));
                    }
                    KeyCode::Down => {
                        self.subscribers
                            .iter()
                            .for_each(|subscriber| subscriber(AppEvent::ScrollDown));
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

        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            subscribers: Vec::new(),
        };

        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sub_results = results.clone();
        tested_event_handler
            .subscribe(move |msg| {
                sub_results.lock().unwrap().push(msg);
            })
            .await;

        let tested_result = tested_event_handler.run().await;

        assert!(tested_result.is_ok());
        assert_eq!(results.lock().unwrap()[0], AppEvent::Quit);
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

        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            subscribers: Vec::new(),
        };

        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sub_results = results.clone();
        tested_event_handler
            .subscribe(move |msg| {
                sub_results.lock().unwrap().push(msg);
            })
            .await;

        let tested_result = tested_event_handler.run().await;

        assert!(tested_result.is_ok());
        assert_eq!(results.lock().unwrap()[0], AppEvent::Character(c));
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

        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            subscribers: Vec::new(),
        };

        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sub_results = results.clone();
        tested_event_handler
            .subscribe(move |msg| {
                sub_results.lock().unwrap().push(msg);
            })
            .await;

        let tested_result = tested_event_handler.run().await;

        assert!(tested_result.is_ok());

        let event = results.lock().unwrap()[0].clone();
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

        let mut tested_event_handler = CrosstermEventsHandler {
            event_stream: stream,
            subscribers: Vec::new(),
        };

        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sub_results = results.clone();
        tested_event_handler
            .subscribe(move |msg| {
                sub_results.lock().unwrap().push(msg);
            })
            .await;

        let tested_result = tested_event_handler.run().await;

        assert!(tested_result.is_ok());
        assert!(results.lock().unwrap().is_empty());
    }
}