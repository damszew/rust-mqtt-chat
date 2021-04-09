use anyhow::Result;

pub mod terminal_renderer;

#[derive(Default, Debug, PartialEq)]
pub struct State;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Renderer {
    fn render(&mut self, state: &State) -> Result<()>;
}
