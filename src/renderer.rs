mod ui;

use anyhow::Result;

pub mod terminal_renderer;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct State {
    pub input_message: String,
    pub cursor: usize,
    pub messages: Vec<String>,
}

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Renderer {
    fn render(&mut self, state: &State) -> Result<()>;
}
