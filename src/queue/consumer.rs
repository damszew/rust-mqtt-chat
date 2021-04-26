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

    use mockall::predicate::*;

    #[tokio::test]
    async fn forward_queue_messages_to_subscribers() {
        let expected_payload = "";

        let expected_event = QueueEvent::Message(Message::new(expected_payload.into()));
        let mut returned_messages = vec![expected_payload.as_bytes().to_owned()].into_iter();

        let mut network_backend_mock = MockNetworkBackend::new();
        network_backend_mock
            .expect_recv()
            .times(2)
            .returning(move || returned_messages.next());

        let (sender1, mut receiver1) = mpsc::channel(1);
        let (sender2, mut receiver2) = mpsc::channel(1);

        let mut tested_consumer = Consumer {
            network_backend: network_backend_mock,
            subscribers: vec![sender1, sender2],
        };

        let tested = async move { tested_consumer.run().await };

        let r = tokio::try_join!(
            tokio::time::timeout(std::time::Duration::from_millis(100), tested),
            tokio::time::timeout(std::time::Duration::from_millis(100), receiver1.recv()),
            tokio::time::timeout(std::time::Duration::from_millis(100), receiver2.recv())
        )
        .unwrap();

        assert!(r.0.is_ok());
        assert_eq!(r.1.unwrap(), expected_event);
        assert_eq!(r.2.unwrap(), expected_event);
    }
}
