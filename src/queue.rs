use tokio::sync::mpsc;

use crate::renderer::Message;

#[derive(Debug, PartialEq)]
pub enum QueueEvent {
    Message(Message),
}

pub trait Queue {
    fn subscribe(&mut self, sender: mpsc::Sender<QueueEvent>);
    fn get_publisher(&self) -> mpsc::Sender<QueueEvent>;
}
