pub mod consumer;
pub mod network_backend;
pub mod publisher;

#[derive(Debug, PartialEq)]
pub enum NetworkEvent {
    Message(Vec<u8>),
}
