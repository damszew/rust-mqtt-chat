pub mod consumer;
pub mod network_backend;
pub mod publisher;

use crate::renderer::Message;

#[derive(Debug, PartialEq)]
pub enum QueueEvent {
    Message(Message),
}
