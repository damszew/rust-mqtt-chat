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
                .map(|subscriber| {
                    subscriber.send(QueueEvent::Message(Message::new(
                        String::from_utf8(message.clone()).expect("Received non UTF8 message"),
                    )))
                })
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
    ] => vec![
        QueueEvent::Message(Message::new("".into())),
    ] ; "Handle empty payload")]
    #[test_case(vec![
        "payload 1".as_bytes().to_owned(),
        "payload 2".as_bytes().to_owned(),
    ] => vec![
        QueueEvent::Message(Message::new("payload 1".into())),
        QueueEvent::Message(Message::new("payload 2".into())),
    ] ; "Handle multiple different messages")]
    #[test_case(vec![
        "the same payload".as_bytes().to_owned(),
        "the same payload".as_bytes().to_owned(),
    ] => vec![
        QueueEvent::Message(Message::new("the same payload".into())),
        QueueEvent::Message(Message::new("the same payload".into())),
    ] ; "Handle multiple similar messages")]
    #[test_case(vec![
        "녹".as_bytes().to_owned(),
        "😎".as_bytes().to_owned(),
    ] => vec![
        QueueEvent::Message(Message::new("녹".into())),
        QueueEvent::Message(Message::new("😎".into())),
    ] ; "Handle non ascii messages")]
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
    async fn forward_queue_messages_to_multiple_subscribers() {
        let payload = "test payload";
        let expected_event = QueueEvent::Message(Message::new(payload.into()));
        let network_messages = vec![payload.as_bytes().to_owned()];

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

    #[tokio::test]
    #[should_panic]
    async fn panic_on_non_utf8_message() {
        let payload = vec![0xC0_u8];
        let network_messages = vec![payload.to_owned()];

        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let (test_subscriber, mut test_receiver) = mpsc::channel(1);

        let mut tested_consumer = Consumer {
            network_backend: network_backend_mock,
            subscribers: vec![test_subscriber],
        };

        let tested_consumer_future = async move { tested_consumer.run().await };

        let _test_result = tokio::try_join!(
            tokio::time::timeout(
                std::time::Duration::from_millis(100),
                tested_consumer_future
            ),
            tokio::time::timeout(std::time::Duration::from_millis(100), test_receiver.recv()),
        )
        .unwrap();
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
