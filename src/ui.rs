use std::io::Write;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::renderer::State;

pub fn draw(frame: &mut Frame<CrosstermBackend<impl Write>>, state: &State, chunk: Rect) {
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

fn draw_input_panel(state: &State, frame: &mut Frame<CrosstermBackend<impl Write>>, chunk: Rect) {
    // let input = Paragraph::new(state.input.as_ref())
    //     .style(match state.input_mode {
    //         InputMode::Normal => Style::default(),
    //         InputMode::Editing => Style::default().fg(Color::Yellow),
    //     })
    //     .block(Block::default().borders(Borders::ALL).title("Input"));

    let input = Paragraph::new("")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, chunk);

    // match state.input_mode {
    //     InputMode::Normal =>
    //         // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
    //         {}

    //     InputMode::Editing => {
    //         // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
    //         frame.set_cursor(
    //             // Put cursor past the end of the input text
    //             chunk.x + state.input.len() as u16 + 1,
    //             // Move one line down, from the border to the input line
    //             chunk.y + 1,
    //         )
    //     }
    // }

    frame.set_cursor(
        // Put cursor past the end of the input text
        chunk.x + 0 as u16 + 1,
        // Move one line down, from the border to the input line
        chunk.y + 1,
    )
}

fn draw_help_msg(state: &State, frame: &mut Frame<CrosstermBackend<impl Write>>, chunk: Rect) {
    // let (msg, style) = match state.input_mode {
    //     InputMode::Normal => (
    //         vec![
    //             Span::raw("Press "),
    //             Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to exit, "),
    //             Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to start editing."),
    //         ],
    //         Style::default().add_modifier(Modifier::RAPID_BLINK),
    //     ),
    //     InputMode::Editing => (
    //         vec![
    //             Span::raw("Press "),
    //             Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to stop editing, "),
    //             Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to record the message"),
    //         ],
    //         Style::default(),
    //     ),
    // };
    let (msg, style) = {
        (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        )
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, chunk);
}

fn draw_messages_panel(
    state: &State,
    frame: &mut Frame<CrosstermBackend<impl Write>>,
    chunk: Rect,
) {
    // let messages = state
    //     .messages
    //     .iter()
    //     .rev()
    //     .map(|m| {
    //         let content = Spans::from(Span::raw(format!("{}", m)));
    //         content
    //     })
    //     .collect::<Vec<_>>();
    let messages = Vec::new();
    let messages =
        Paragraph::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    frame.render_widget(messages, chunk);
}
