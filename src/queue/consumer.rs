use anyhow::Result;
use tokio::sync::mpsc;

use super::{network_backend::NetworkBackend, QueueEvent};
use crate::renderer::Message;

pub struct Consumer<Q> {
    network_backend: Q,
    subscribers: Vec<mpsc::Sender<QueueEvent>>,
}
impl<Q> Consumer<Q>
where
    Q: NetworkBackend,
{
    pub async fn subscribe(&mut self, subscriber: mpsc::Sender<QueueEvent>) -> Result<()> {
        self.subscribers.push(subscriber);
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.network_backend.recv().await {
            let futures = self
                .subscribers
                .iter()
                .map(|subscriber| subscriber.send(QueueEvent::Message(Message::new("".into()))))
                .collect::<Vec<_>>();

            futures::future::join_all(futures).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::network_backend::MockNetworkBackend, *};

    use test_case::test_case;

    #[test_case(vec![
        "".as_bytes().to_owned(),
        "".as_bytes().to_owned(),
    ] => vec![
        QueueEvent::Message(Message::new("".into())),
        QueueEvent::Message(Message::new("".into())),
    ] ; "test name")]
    #[tokio::test]
    async fn receive_couple_messages(network_messages: Vec<Vec<u8>>) -> Vec<QueueEvent> {
        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let (test_subscriber, mut test_receiver) = mpsc::channel(1);

        let mut tested_consumer = Consumer {
            network_backend: network_backend_mock,
            subscribers: vec![test_subscriber],
        };

        let tested_consumer_future = async move { tested_consumer.run().await };
        let test_receiver_future = async {
            let mut result = Vec::new();
            while let Some(msg) = test_receiver.recv().await {
                result.push(msg);
            }
            result
        };

        let test_result = tokio::try_join!(
            tokio::time::timeout(
                std::time::Duration::from_millis(100),
                tested_consumer_future
            ),
            tokio::time::timeout(std::time::Duration::from_millis(100), test_receiver_future),
        )
        .unwrap();

        assert!(test_result.0.is_ok());

        test_result.1
    }

    #[tokio::test]
    async fn forward_queue_messages_to_subscribers() {
        let expected_event = QueueEvent::Message(Message::new("".into()));
        let network_messages = vec!["".as_bytes().to_owned()];

        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let (test_subscriber1, mut test_receiver1) = mpsc::channel(1);
        let (test_subscriber2, mut test_receiver2) = mpsc::channel(1);

        let mut tested_consumer = Consumer {
            network_backend: network_backend_mock,
            subscribers: vec![test_subscriber1, test_subscriber2],
        };

        let tested_consumer_future = async move { tested_consumer.run().await };

        let test_result = tokio::try_join!(
            tokio::time::timeout(
                std::time::Duration::from_millis(100),
                tested_consumer_future
            ),
            tokio::time::timeout(std::time::Duration::from_millis(100), test_receiver1.recv()),
            tokio::time::timeout(std::time::Duration::from_millis(100), test_receiver2.recv())
        )
        .unwrap();

        assert!(test_result.0.is_ok());
        assert_eq!(test_result.1.unwrap(), expected_event);
        assert_eq!(test_result.2.unwrap(), expected_event);
    }

    fn setup_network_mock(mut network_messages: std::vec::IntoIter<Vec<u8>>) -> MockNetworkBackend {
        let mut network_backend_mock = MockNetworkBackend::new();
        network_backend_mock
            .expect_recv()
            .times(network_messages.len() + 1)
            .returning(move || network_messages.next());
        network_backend_mock
    }
}
