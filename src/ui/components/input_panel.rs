use crossterm::event::KeyEvent;
use tui::{backend::Backend, layout::Rect, style, widgets, Frame};

#[derive(Clone, Default, Debug)]
pub struct InputPanel {
    input_message: String,
    cursor: usize,
}

impl InputPanel {
    pub fn new() -> Self {
        Self {
            input_message: String::new(),
            cursor: 0,
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
                dbg!(&self.input_message);
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
