use std::sync::{Arc, Mutex};

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

use crate::chat_room::{ChatMessage, ChatRoom};

#[derive(Clone, Default, Debug)]
pub struct MessagesPanel {
    messages: Arc<Mutex<Vec<ChatMessage>>>,
}

impl MessagesPanel {
    pub fn new(chat_room: &mut impl ChatRoom) -> Self {
        let messages = Arc::new(Mutex::new(Vec::new()));

        chat_room.on_message({
            let messages = messages.clone();
            move |msg| {
                messages.lock().expect("Poisoned mutex").push(msg);
            }
        });

        Self { messages }
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, chunk: Rect) {
        let messages = self
            .messages
            .lock()
            .expect("Poisoned mutex")
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
