use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::renderer::State;

pub fn draw(frame: &mut Frame<impl Backend>, state: &State, chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(chunk);

    draw_messages_panel(state, frame, chunks[0]);
    draw_help_msg(state, frame, chunks[1]);
    draw_input_panel(state, frame, chunks[2]);
}

fn draw_input_panel(state: &State, frame: &mut Frame<impl Backend>, chunk: Rect) {
    let input = Paragraph::new(state.input_message.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, chunk);

    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
    frame.set_cursor(
        // Put cursor past the end of the input text
        chunk.x + state.cursor as u16 + 1,
        // Move one line down, from the border to the input line
        chunk.y + 1,
    );
}

fn draw_help_msg(_state: &State, frame: &mut Frame<impl Backend>, chunk: Rect) {
    let msg = {
        vec![
            Span::raw("Press "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to exit, "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to send the message"),
        ]
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(Style::default());
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, chunk);
}

fn draw_messages_panel(state: &State, frame: &mut Frame<impl Backend>, chunk: Rect) {
    let messages = state
        .messages
        .iter()
        .map(|message| {
            let content = Spans::from(vec![
                Span::raw(message.time.format("%H:%M:%S ").to_string()),
                Span::styled(
                    message.user.clone(),
                    Style::default()
                        .fg(get_rbg(&message.user))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::raw(message.msg.clone()),
            ]);
            content
        })
        .collect::<Vec<_>>();

    let messages =
        Paragraph::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    frame.render_widget(messages, chunk);
}

fn get_rbg(data: &str) -> Color {
    let mut rng: Pcg64 = Seeder::from(data).make_rng();
    let (r, g, b) = rng.gen();

    Color::Rgb(r, g, b)
}
