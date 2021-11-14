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
