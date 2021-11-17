use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::Paragraph,
    Frame,
};

#[derive(Clone, Default, Debug)]
pub struct HelpMsg {}

impl HelpMsg {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, chunk: Rect) {
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
}
