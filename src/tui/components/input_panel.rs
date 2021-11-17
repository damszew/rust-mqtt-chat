use crossterm::event::KeyEvent;
use tui::{backend::Backend, layout::Rect, style, widgets, Frame};

use crate::chat_room::ChatRoom;

pub struct InputPanel<C> {
    input_message: String,
    cursor: usize,
    chat_room: C,
}

impl<C> InputPanel<C>
where
    C: ChatRoom,
{
    pub fn new(chat_room: C) -> Self {
        Self {
            input_message: String::new(),
            cursor: 0,
            chat_room,
        }
    }

    pub async fn update(&mut self, event: KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Char(ch) => {
                let cursor = self.cursor;
                self.input_message.insert(cursor, ch);
                self.cursor += 1;
            }

            crossterm::event::KeyCode::Enter => {
                let message = self.input_message.drain(..).collect::<String>();
                if !message.is_empty() {
                    self.chat_room.send(message).await.unwrap();
                    self.cursor = 0;
                }
            }

            crossterm::event::KeyCode::Delete => {
                if self.cursor < self.input_message.len() {
                    let cursor = self.cursor;
                    self.input_message.remove(cursor);
                }
            }
            crossterm::event::KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    let cursor = self.cursor;
                    self.input_message.remove(cursor);
                }
            }

            crossterm::event::KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.cursor < self.input_message.len() {
                    self.cursor += 1;
                }
            }

            crossterm::event::KeyCode::Home => {
                self.cursor = 0;
            }
            crossterm::event::KeyCode::End => {
                self.cursor = self.input_message.len();
            }
            _ => (),
        }
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, chunk: Rect) {
        let input = widgets::Paragraph::new(self.input_message.as_ref())
            .style(style::Style::default())
            .block(
                widgets::Block::default()
                    .borders(widgets::Borders::ALL)
                    .title("Input"),
            );

        frame.render_widget(input, chunk);

        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        frame.set_cursor(
            // Put cursor past the end of the input text
            chunk.x + self.cursor as u16 + 1,
            // Move one line down, from the border to the input line
            chunk.y + 1,
        );
    }
}

#[cfg(test)]
mod tests {
    use mockall::{predicate::eq, Sequence};
    use test_case::test_case;

    use crossterm::event::{KeyCode, KeyModifiers};

    use super::*;

    use crate::chat_room::MockChatRoom;

    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "me"
        ; "typed_message")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "m"
        ; "remove last character")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "me"
        ; "remove last on empty message")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "mhe"
        ; "insert letter inside msg")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "e"
        ; "remove letter inside msg")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "m"
        ; "delete letter inside msg")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "me"
        ; "delete at end does nothing")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "hmed"
        ; "jump to start and end of msg")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "mhed"
        ; "travel msg with arrow keys")]
    #[test_case(
        vec![
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ],
        "me"
        ; "arrow keys on empty msg does nothing")]
    #[tokio::test]
    async fn should_send_typed_msg(events: Vec<KeyEvent>, expected_msg: &str) {
        let mut chat_room_mock = MockChatRoom::new();
        chat_room_mock
            .expect_send()
            .times(1)
            .with(eq(expected_msg.to_string()))
            .returning(|_| Ok(()));

        let mut sut = InputPanel::new(chat_room_mock);

        for event in events {
            sut.update(event).await;
        }
    }

    #[tokio::test]
    async fn should_not_send_empty_msg() {
        let mut chat_room_mock = MockChatRoom::new();
        chat_room_mock.expect_send().never().returning(|_| Ok(()));

        let mut sut = InputPanel::new(chat_room_mock);

        sut.update(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .await;
    }

    #[tokio::test]
    async fn should_write_2nd_msg_on_empty_buffer() {
        let events = vec![
            KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ];

        let mut seq = Sequence::new();
        let mut chat_room_mock = MockChatRoom::new();
        chat_room_mock
            .expect_send()
            .times(1)
            .in_sequence(&mut seq)
            .with(eq("me".to_string()))
            .returning(|_| Ok(()));
        chat_room_mock
            .expect_send()
            .times(1)
            .in_sequence(&mut seq)
            .with(eq("too".to_string()))
            .returning(|_| Ok(()));

        let mut sut = InputPanel::new(chat_room_mock);

        for event in events {
            sut.update(event).await;
        }
    }
}
