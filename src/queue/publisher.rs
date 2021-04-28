use anyhow::Result;
use tokio::sync::mpsc;

use super::{network_backend::NetworkBackend, NetworkEvent};

pub struct Publisher<Q> {
    network_backend: Q,
    publishers: mpsc::Receiver<NetworkEvent>,
}

impl<Q> Publisher<Q>
where
    Q: NetworkBackend,
{
    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.publishers.recv().await {
            match message {
                NetworkEvent::Message(msg) => {
                    self.network_backend.send(msg).await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::network_backend::MockNetworkBackend, *};

    use mockall::predicate::*;
    use test_case::test_case;

    #[test_case(
        vec![
            NetworkEvent::Message("".into()),
        ],
        vec![
            "".as_bytes().to_owned(),
        ]
        ; "Handle empty payload")]
    #[test_case(
        vec![
            NetworkEvent::Message("payload 1".into()),
            NetworkEvent::Message("payload 2".into()),
        ],
        vec![
            "payload 1".as_bytes().to_owned(),
            "payload 2".as_bytes().to_owned(),
        ]
        ; "Publish messages in proper order")]
    #[test_case(
        vec![
            NetworkEvent::Message("the same payload".into()),
            NetworkEvent::Message("the same payload".into()),
        ],
        vec![
            "the same payload".as_bytes().to_owned(),
            "the same payload".as_bytes().to_owned(),
        ]
        ; "Publish even repeated messages")]
    #[test_case(
        vec![
            NetworkEvent::Message("녹".into()),
            NetworkEvent::Message("😎".into()),
        ],
        vec![
            "녹".as_bytes().to_owned(),
            "😎".as_bytes().to_owned(),
        ]
        ; "Handle non ascii messages")]
    #[tokio::test]
    async fn publish_couple_messages(
        published_messages: Vec<NetworkEvent>,
        expected_messages: Vec<Vec<u8>>,
    ) {
        let mut network_backend_mock = MockNetworkBackend::new();
        for expected_message in expected_messages {
            network_backend_mock
                .expect_send()
                .times(1)
                .with(eq(expected_message))
                .returning(move |_| Ok(()));
        }

        let (test_sender, receiver) = mpsc::channel(1);
        let mut tested_publisher = Publisher {
            network_backend: network_backend_mock,
            publishers: receiver,
        };

        let send = async move {
            for event in published_messages {
                test_sender.send(event).await?;
            }
            Ok::<_, anyhow::Error>(())
        };
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
