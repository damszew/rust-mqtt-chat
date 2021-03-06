use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::chat_room::ChatRoom;

use super::{help_msg::HelpMsg, input_panel::InputPanel, messages_panel::MessagesPanel};

pub struct MainView<C> {
    msg_panel: MessagesPanel<C>,
    help_msg: HelpMsg,
    input_panel: InputPanel<C>,
}

impl<C> MainView<C>
where
    C: ChatRoom + Clone,
{
    pub fn new(chat_room: C) -> Self {
        let msg_panel = MessagesPanel::new(chat_room.clone());
        let help_msg = HelpMsg::new();
        let input_panel = InputPanel::new(chat_room);

        Self {
            msg_panel,
            help_msg,
            input_panel,
        }
    }

    pub async fn update(&mut self, event: KeyEvent) {
        self.input_panel.update(event).await
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, chunk: Rect) {
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

        self.msg_panel.draw(frame, chunks[0]);
        self.help_msg.draw(frame, chunks[1]);
        self.input_panel.draw(frame, chunks[2]);
    }
}
