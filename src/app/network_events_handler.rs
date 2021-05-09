use std::sync::{Arc, Mutex};

use crate::{network::NetworkEvent, renderer::State};

pub struct NetworkEventsHandler {
    state: Arc<Mutex<State>>,
}
impl NetworkEventsHandler {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        Self { state }
    }
    pub fn handle(&self, message: NetworkEvent) {
        match message {
            NetworkEvent::Message(payload) => {
                let message = serde_json::from_slice(&payload).expect("Received invalid message");
                self.state.lock().unwrap().messages.push(message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::renderer::Message;

    use chrono::{DateTime, Local};
    use test_case::test_case;

    #[test_case(
        State::default(),
        vec![
            NetworkEvent::Message(r#"{"user":"Chef","msg":"Hi","time":"2021-05-09T09:00:00+02:00"}"#.into())
        ]
        =>
        State {
            messages: vec![
                Message {
                    user: "Chef".into(),
                    msg: "Hi".into(),
                    time: DateTime::parse_from_rfc3339("2021-05-09T09:00:00+02:00")
                        .unwrap()
                        .with_timezone(&Local),
                },
            ],
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
