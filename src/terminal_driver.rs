use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::{FutureExt, StreamExt};

use crate::ui::terminal_ui::TerminalUi;

use std::io::Write;

use anyhow::Result;
use crossterm::{terminal, ExecutableCommand};
use tui::{backend::CrosstermBackend, Terminal};

pub struct TerminalDriver<W: Write> {
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> TerminalDriver<W> {
    pub fn new(mut out: W) -> Result<TerminalDriver<W>> {
        terminal::enable_raw_mode()?;
        out.execute(terminal::EnterAlternateScreen)?;

        Ok(TerminalDriver {
            terminal: Terminal::new(CrosstermBackend::new(out))?,
        })
    }

    pub async fn run(&mut self, mut ui: TerminalUi) -> Result<()> {
        let mut event_stream = EventStream::new();
        let timeout = std::time::Duration::from_millis(15);

        loop {
            self.render(&ui)?;

            if let Ok(event) = tokio::time::timeout(timeout, event_stream.next().fuse()).await {
                let event = event.ok_or_else(|| anyhow::anyhow!("Empty events queue"))??;
                if let Event::Key(event) = event {
                    if quit_event_happened(event) {
                        break;
                    }

                    ui.update(event).await;
                }
            };
        }

        Ok(())
    }

    fn render(&mut self, ui: &TerminalUi) -> Result<()> {
        self.terminal.draw(|frame| ui.draw(frame, frame.size()))?;

        Ok(())
    }
}

impl<W: Write> Drop for TerminalDriver<W> {
    fn drop(&mut self) {
        self.terminal
            .backend_mut()
            .execute(terminal::LeaveAlternateScreen)
            .expect("Could not execute to stdout");
        terminal::disable_raw_mode().expect("Terminal doesn't support to disable raw mode");
    }
}

fn quit_event_happened(event: KeyEvent) -> bool {
    event.code == KeyCode::Esc
        || event.code == KeyCode::Char('c') && event.modifiers == KeyModifiers::CONTROL
}
