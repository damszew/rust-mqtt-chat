use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::chat_room::ChatRoom;

#[derive(Clone, Default, Debug)]
pub struct MessagesPanel<C> {
    chat_room: C,
}

impl<C> MessagesPanel<C>
where
    C: ChatRoom,
{
    pub fn new(chat_room: C) -> Self {
        Self { chat_room }
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, chunk: Rect) {
        let messages = self
            .chat_room
            .get_messages()
            .iter()
            .map(|message| {
                Spans::from(vec![
                    Span::raw(message.time.format("%H:%M:%S ").to_string()),
                    Span::styled(
                        message.user.clone(),
                        Style::default()
                            .fg(get_rbg(&message.user))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::raw(message.msg.clone()),
                ])
            })
            .collect::<Vec<_>>();

        let messages = Paragraph::new(messages)
            .block(Block::default().borders(Borders::ALL).title("Messages"));
        frame.render_widget(messages, chunk);
    }
}

fn get_rbg(data: &str) -> Color {
    let mut rng: Pcg64 = Seeder::from(data).make_rng();
    let (r, g, b) = rng.gen();

    Color::Rgb(r, g, b)
}
