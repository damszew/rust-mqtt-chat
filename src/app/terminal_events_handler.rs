use std::sync::{Arc, Mutex};

use crate::{
    renderer::{Message, State},
    AppEvent,
};

pub struct TerminalEventsHandler {
    state: Arc<Mutex<State>>,
}
impl TerminalEventsHandler {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        Self { state }
    }
    pub fn handle(&self, message: AppEvent) {
        let mut state = self.state.lock().unwrap();
        match message {
            AppEvent::Quit => {
                state.quit = true;
            }
            AppEvent::Character(ch) => {
                let cursor = state.cursor;
                state.input_message.insert(cursor, ch);
                state.cursor += 1;
            }
            AppEvent::Accept => {
                let message = state.input_message.drain(..).collect::<String>();
                let message = Message::new(message);

                state.cursor = 0;
                state.messages.push(message);
                // self.network_publisher
                //     .send(NetworkEvent::Message(message.as_bytes().to_owned()))
                //     .await?;
            }
            AppEvent::Remove => {
                if state.cursor < state.input_message.len() {
                    let cursor = state.cursor;
                    state.input_message.remove(cursor);
                }
            }
            AppEvent::RemoveLast => {
                if state.cursor > 0 {
                    state.cursor -= 1;
                    let cursor = state.cursor;
                    state.input_message.remove(cursor);
                }
            }
            AppEvent::CursorStart => {
                state.cursor = 0;
            }
            AppEvent::CursorEnd => {
                state.cursor = state.input_message.len();
            }
            AppEvent::CursorRight => {
                if state.cursor < state.input_message.len() {
                    state.cursor += 1;
                }
            }
            AppEvent::CursorLeft => {
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
    use super::*;

    use test_case::test_case;

    #[test_case(
        State::default(),
        vec![
            AppEvent::Character('c')
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
            AppEvent::RemoveLast,
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
            AppEvent::RemoveLast,
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
            AppEvent::RemoveLast,
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
            AppEvent::Remove,
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
            AppEvent::Remove,
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
            AppEvent::Character('!'),
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
            AppEvent::CursorLeft,
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
            AppEvent::CursorRight,
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
            AppEvent::CursorStart,
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
            AppEvent::CursorStart,
            AppEvent::CursorLeft,
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
            AppEvent::CursorEnd,
            AppEvent::CursorRight,
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
            AppEvent::Quit,
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
        events: Vec<AppEvent>,
    ) -> State {
        let state = Arc::new(Mutex::new(init_state));
        let tested_handler = TerminalEventsHandler::new(state.clone());

        for event in events {
            tested_handler.handle(event);
        }

        let result = state.lock().unwrap().clone();
        result
    }
}
