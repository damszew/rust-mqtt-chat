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

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case(
        State::default(),
        vec![
            NetworkEvent::Message("Hi".into())
        ]
        =>
        State {
            messages: vec![Message::new("Hi".into())],
            ..Default::default()
        }
        ; "simple message")]
    #[tokio::test]
    async fn update_state_based_on_network_events(
        init_state: State,
        events: Vec<NetworkEvent>,
    ) -> State {
        let state = Arc::new(Mutex::new(init_state));
        let tested_handler = NetworkEventsHandler::new(state.clone());

        for event in events {
            tested_handler.handle(event);
        }

        let result = state.lock().unwrap().clone();
        result
    }
}
