use std::sync::{Arc, Mutex};

use crate::{
    network::NetworkEvent,
    renderer::{Message, State},
};

pub struct NetworkEventsHandler {
    state: Arc<Mutex<State>>,
}
impl NetworkEventsHandler {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        Self { state }
    }
    pub fn handle(&self, message: NetworkEvent) {
        match message {
            NetworkEvent::Message(payload) => self
                .state
                .lock()
                .unwrap()
                .messages
                .push(Message::new(String::from_utf8(payload).unwrap())),
        }
    }
}
