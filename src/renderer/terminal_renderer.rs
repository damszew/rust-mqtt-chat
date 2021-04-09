use std::io::Write;

use anyhow::Result;
use crossterm::{terminal, ExecutableCommand};
use tui::{backend::CrosstermBackend, Terminal};

use crate::ui;

use super::{Renderer, State};

pub struct TerminalRenderer<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> TerminalRenderer<W> {
    pub fn new(mut out: W) -> Result<TerminalRenderer<W>> {
        terminal::enable_raw_mode()?;
        out.execute(terminal::EnterAlternateScreen)?;

        Ok(TerminalRenderer {
            terminal: Terminal::new(CrosstermBackend::new(out))?,
        })
    }
}

impl<W: Write> Renderer for TerminalRenderer<W> {
    fn render(&mut self, state: &State) -> Result<()> {
        self.terminal
            .draw(|frame| ui::draw(frame, state, frame.size()))?;

        Ok(())
    }
}

impl<W: Write> Drop for TerminalRenderer<W> {
    fn drop(&mut self) {
        self.terminal
            .backend_mut()
            .execute(terminal::LeaveAlternateScreen)
            .expect("Could not execute to stdout");
        terminal::disable_raw_mode().expect("Terminal doesn't support to disable raw mode");
    }
}
