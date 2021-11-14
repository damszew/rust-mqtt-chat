use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::components::{
    help_msg::HelpMsg, input_panel::InputPanel, messages_panel::MessagesPanel,
};

#[derive(Clone, Default, Debug)]
pub struct TerminalUi {
    msg_panel: MessagesPanel,
    help_msg: HelpMsg,
    input_panel: InputPanel,
}

impl TerminalUi {
    pub fn new() -> Self {
        let msg_panel = MessagesPanel::new();
        let help_msg = HelpMsg::new();
        let input_panel = InputPanel::new();

        Self {
            msg_panel,
            help_msg,
            input_panel,
        }
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
