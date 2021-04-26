use anyhow::Result;
use tokio::sync::mpsc;

use super::{network_backend::NetworkBackend, QueueEvent};

pub struct Publisher<Q> {
    network_backend: Q,
    publishers: mpsc::Receiver<QueueEvent>,
}

impl<Q> Publisher<Q>
where
    Q: NetworkBackend,
{
    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.publishers.recv().await {
            self.network_backend.send(Vec::new()).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::network_backend::MockNetworkBackend, *};
    use crate::renderer::Message;

    use mockall::predicate::*;

    #[tokio::test]
    async fn forward_messages_from_publishers_to_queue() {
        let event_to_publish = QueueEvent::Message(Message::new("".into()));

        let mut network_backend_mock = MockNetworkBackend::new();
        network_backend_mock.expect_recv().returning(|| None);
        network_backend_mock
            .expect_send()
            .times(1)
            .with(eq(Vec::new()))
            .returning(move |_| Ok(()));

        let (sender, receiver) = mpsc::channel(1);
        let mut tested_publisher = Publisher {
            network_backend: network_backend_mock,
            publishers: receiver,
        };

        let send = async move { sender.send(event_to_publish).await };
        let tested = async move { tested_publisher.run().await };

        let r = tokio::try_join!(
            tokio::time::timeout(std::time::Duration::from_millis(100), tested),
            tokio::time::timeout(std::time::Duration::from_millis(100), send)
        )
        .unwrap();

        assert!(r.0.is_ok());
        assert!(r.1.is_ok());
    }
}
