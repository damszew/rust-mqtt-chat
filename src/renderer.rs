#[derive(Debug, PartialEq)]
pub struct State;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Renderer {
    fn render(&mut self, state: &State);
}
