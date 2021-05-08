use std::sync::{Arc, Mutex};

use crate::{
    events_publisher::EventsPublisher, network::NetworkEvent, renderer::State, TerminalEvent,
};

pub struct TerminalEventsHandler<EP>
where
    EP: EventsPublisher<Message = NetworkEvent>,
{
    state: Arc<Mutex<State>>,
    events_publisher: EP,
}
impl<EP> TerminalEventsHandler<EP>
where
    EP: EventsPublisher<Message = NetworkEvent>,
{
    pub fn new(state: Arc<Mutex<State>>, events_publisher: EP) -> Self {
        Self {
            state,
            events_publisher,
        }
    }
    pub fn handle(&self, message: TerminalEvent) {
        let mut state = self.state.lock().unwrap();
        match message {
            TerminalEvent::Quit => {
                state.quit = true;
            }
            TerminalEvent::Character(ch) => {
                let cursor = state.cursor;
                state.input_message.insert(cursor, ch);
                state.cursor += 1;
            }
            TerminalEvent::Accept => {
                let message = state.input_message.drain(..).collect::<String>();

                self.events_publisher
                    .publish(NetworkEvent::Message(message.as_bytes().to_owned()))
                    .unwrap();

                state.cursor = 0;
            }
            TerminalEvent::Remove => {
                if state.cursor < state.input_message.len() {
                    let cursor = state.cursor;
                    state.input_message.remove(cursor);
                }
            }
            TerminalEvent::RemoveLast => {
                if state.cursor > 0 {
                    state.cursor -= 1;
                    let cursor = state.cursor;
                    state.input_message.remove(cursor);
                }
            }
            TerminalEvent::CursorStart => {
                state.cursor = 0;
            }
            TerminalEvent::CursorEnd => {
                state.cursor = state.input_message.len();
            }
            TerminalEvent::CursorRight => {
                if state.cursor < state.input_message.len() {
                    state.cursor += 1;
                }
            }
            TerminalEvent::CursorLeft => {
                if state.cursor > 0 {
                    state.cursor -= 1;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{events_publisher::EventsPublisher, network::NetworkEvent};

    use super::*;

    use mockall::predicate::*;
    use test_case::test_case;

    #[test_case(
        State::default(),
        vec![
            TerminalEvent::Character('c')
        ]
        =>
        State {
            input_message: "c".into(),
            cursor: 1,
            ..Default::default()
        }
        ; "single character")]
    #[test_case(
        State {
            input_message: "me".into(),
            cursor: 2,
            ..Default::default()
        },
        vec![
            TerminalEvent::RemoveLast,
        ]
        =>
        State {
            input_message: "m".into(),
            cursor: 1,
            ..Default::default()
        }
        ; "remove last character")]
    #[test_case(
        State::default(),
        vec![
            TerminalEvent::RemoveLast,
        ]
        =>
        State {
            input_message: "".into(),
            cursor: 0,
            ..Default::default()
        }
        ; "remove on empty buffer")]
    #[test_case(
        State {
            input_message: "some->message".into(),
            cursor: 6,
            ..Default::default()
        },
        vec![
            TerminalEvent::RemoveLast,
        ]
        =>
        State {
            input_message: "some-message".into(),
            cursor: 5,
            ..Default::default()
        }
        ; "remove when inside message")]
    #[test_case(
        State {
            input_message: "some->message".into(),
            cursor: 5,
            ..Default::default()
        },
        vec![
            TerminalEvent::Remove,
        ]
        =>
        State {
            input_message: "some-message".into(),
            cursor: 5,
            ..Default::default()
        }
        ; "delete when inside message")]
    #[test_case(
        State {
            input_message: "some".into(),
            cursor: 4,
            ..Default::default()
        },
        vec![
            TerminalEvent::Remove,
        ]
        =>
        State {
            input_message: "some".into(),
            cursor: 4,
            ..Default::default()
        }
        ; "delete at the end")]
    #[test_case(
        State {
            input_message: "some message".into(),
            cursor: 4,
            ..Default::default()
        },
        vec![
            TerminalEvent::Character('!'),
        ]
        =>
        State {
            input_message: "some! message".into(),
            cursor: 5,
            ..Default::default()
        }
        ; "insert char inside message")]
    #[test_case(
        State {
            input_message: "some message".into(),
            cursor: 5,
            ..Default::default()
        },
        vec![
            TerminalEvent::CursorLeft,
        ]
        =>
        State {
            input_message: "some message".into(),
            cursor: 4,
            ..Default::default()
        }
        ; "cursor left")]
    #[test_case(
        State {
            input_message: "some message".into(),
            cursor: 5,
            ..Default::default()
        },
        vec![
            TerminalEvent::CursorRight,
        ]
        =>
        State {
            input_message: "some message".into(),
            cursor: 6,
            ..Default::default()
        }
        ; "cursor right")]
    #[test_case(
        State {
            input_message: "some message".into(),
            cursor: 5,
            ..Default::default()
        },
        vec![
            TerminalEvent::CursorStart,
        ]
        =>
        State {
            input_message: "some message".into(),
            cursor: 0,
            ..Default::default()
        }
        ; "cursor start")]
    #[test_case(
        State {
            input_message: "some message".into(),
            cursor: 0,
            ..Default::default()
        },
        vec![
            TerminalEvent::CursorStart,
            TerminalEvent::CursorLeft,
        ]
        =>
        State {
            input_message: "some message".into(),
            cursor: 0,
            ..Default::default()
        }
        ; "cursor pos can not be negative")]
    #[test_case(
        State {
            input_message: "some message".into(),
            cursor: 0,
            ..Default::default()
        },
        vec![
            TerminalEvent::CursorEnd,
            TerminalEvent::CursorRight,
        ]
        =>
        State {
            input_message: "some message".into(),
            cursor: 12,
            ..Default::default()
        }
        ; "do not exceed input message")]
    #[test_case(
        State::default(),
        vec![
            TerminalEvent::Quit,
        ]
        =>
        State {
            quit: true,
            ..Default::default()
        }
        ; "quit")]
    #[tokio::test]
    async fn update_state_based_on_terminal_events(
        init_state: State,
        events: Vec<TerminalEvent>,
    ) -> State {
        let state = Arc::new(Mutex::new(init_state));

        let events_publisher_mock = MockEventsPublisher::new();
        let tested_handler = TerminalEventsHandler::new(state.clone(), events_publisher_mock);

        for event in events {
            tested_handler.handle(event);
        }

        let result = state.lock().unwrap().clone();
        result
    }

    #[tokio::test]
    async fn send_message_on_accept() {
        let init_state = State {
            input_message: "some message".into(),
            cursor: 12,
            ..Default::default()
        };
        let events = vec![TerminalEvent::Accept];
        let expected_state = State {
            input_message: "".into(),
            cursor: 0,
            ..Default::default()
        };

        let state = Arc::new(Mutex::new(init_state));

        let mut events_publisher_mock = MockEventsPublisher::new();
        events_publisher_mock
            .expect_publish()
            .with(eq(NetworkEvent::Message("some message".into())))
            .once()
            .returning(|_| Ok(()));

        let tested_handler = TerminalEventsHandler::new(state.clone(), events_publisher_mock);

        for event in events {
            tested_handler.handle(event);
        }

        let result = state.lock().unwrap().clone();
        assert_eq!(result, expected_state);
    }

    mockall::mock! {
        EventsPublisher {}

        impl EventsPublisher for EventsPublisher {
            type Message = NetworkEvent;

            fn publish(&self, message: NetworkEvent) -> anyhow::Result<()>;
        }
    }
}
