use anyhow::Result;
use futures::{channel::mpsc::Receiver, StreamExt};
use paho_mqtt::Message;

use crate::network::NetworkEvent;

use super::EventsReader;

pub struct MqttEventsReader {
    pub receiver: Receiver<Option<Message>>,
    pub subscribers: Vec<Box<dyn Fn(NetworkEvent) -> () + Send + 'static>>,
}

impl MqttEventsReader {
    pub fn new(receiver: Receiver<Option<Message>>) -> Self {
        Self {
            receiver,
            subscribers: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl EventsReader for MqttEventsReader {
    type Message = NetworkEvent;

    async fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(Self::Message) -> () + Send + 'static,
    {
        self.subscribers.push(Box::new(callback));
    }

    async fn run(&mut self) -> Result<()> {
        while let Some(Some(message)) = self.receiver.next().await {
            let message = message.payload().to_owned();

            self.subscribers
                .iter()
                .for_each(|subscriber| subscriber(NetworkEvent::Message(message.clone())));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    use test_case::test_case;

    #[test_case(vec![
        "".as_bytes().to_owned(),
    ], vec![
        NetworkEvent::Message("".into()),
    ] ; "Handle empty payload")]
    #[test_case(vec![
        "payload 1".as_bytes().to_owned(),
        "payload 2".as_bytes().to_owned(),
    ], vec![
        NetworkEvent::Message("payload 1".into()),
        NetworkEvent::Message("payload 2".into()),
    ] ; "Handle multiple different messages")]
    #[test_case(vec![
        "the same payload".as_bytes().to_owned(),
        "the same payload".as_bytes().to_owned(),
    ], vec![
        NetworkEvent::Message("the same payload".into()),
        NetworkEvent::Message("the same payload".into()),
    ] ; "Handle multiple similar messages")]
    #[test_case(vec![
        "녹".as_bytes().to_owned(),
        "😎".as_bytes().to_owned(),
    ], vec![
        NetworkEvent::Message("녹".into()),
        NetworkEvent::Message("😎".into()),
    ] ; "Handle non ascii messages")]
    #[tokio::test]
    async fn receive_couple_messages(
        network_messages: Vec<Vec<u8>>,
        expected_messages: Vec<NetworkEvent>,
    ) {
        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let mut tested_consumer = MqttEventsReader::new(network_backend_mock);

        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sub_results = results.clone();
        tested_consumer
            .subscribe(move |msg| {
                sub_results.lock().unwrap().push(msg);
            })
            .await;

        let test_result = tested_consumer.run().await;

        assert!(test_result.is_ok());
        assert_eq!((*results.lock().unwrap()), expected_messages);
    }

    #[tokio::test]
    async fn forward_queue_messages_to_multiple_subscribers() {
        let payload = "test payload";
        let expected_event = NetworkEvent::Message(payload.into());
        let network_messages = vec![payload.as_bytes().to_owned()];

        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let mut tested_consumer = MqttEventsReader::new(network_backend_mock);
        let (test_subscriber1, test_receiver1) = mpsc::channel();
        tested_consumer
            .subscribe(move |msg| {
                test_subscriber1.send(msg).unwrap();
            })
            .await;

        let (test_subscriber2, test_receiver2) = mpsc::channel();
        tested_consumer
            .subscribe(move |msg| {
                test_subscriber2.send(msg).unwrap();
            })
            .await;

        let tested_consumer_future = async move { tested_consumer.run().await };

        let test_result = tokio::try_join!(
            tokio::time::timeout(
                std::time::Duration::from_millis(100),
                tested_consumer_future
            ),
            tokio::time::timeout(std::time::Duration::from_millis(100), async {
                test_receiver1.recv()
            }),
            tokio::time::timeout(std::time::Duration::from_millis(100), async {
                test_receiver2.recv()
            })
        )
        .unwrap();

        assert!(test_result.0.is_ok());
        assert_eq!(test_result.1.unwrap(), expected_event);
        assert_eq!(test_result.2.unwrap(), expected_event);
    }

    fn setup_network_mock(
        network_messages: std::vec::IntoIter<Vec<u8>>,
    ) -> futures::channel::mpsc::Receiver<Option<Message>> {
        let (mut sender, receiver) = futures::channel::mpsc::channel(network_messages.len() + 1);
        for m in network_messages {
            let mqtt_message = Message::new("topic", m, 0);
            sender.try_send(Some(mqtt_message)).unwrap();
        }
        sender.try_send(None).unwrap();

        receiver
    }
}
