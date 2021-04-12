use anyhow::Result;

pub mod terminal_renderer;

#[derive(Default, Debug, PartialEq)]
pub struct State {
    pub input_message: String,
    pub messages: Vec<String>,
}

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Renderer {
    fn render(&mut self, state: &State) -> Result<()>;
}
