pub mod queue_backend;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::renderer::Message;

use self::queue_backend::{BackendMessage, QueueBackend};

#[derive(Debug, PartialEq)]
pub enum QueueEvent {
    Message(Message),
}

struct Queue<Q> {
    queue_backend: Q,
    subscribers: Vec<mpsc::Sender<QueueEvent>>,
    publisher_recv: mpsc::Receiver<QueueEvent>,
}

impl<Q> Queue<Q>
where
    Q: QueueBackend,
{
    pub async fn new(mut queue_backend: Q) -> Result<(Self, mpsc::Sender<QueueEvent>)> {
        queue_backend.connect().await?;
        queue_backend.subscribe("topic").await?;

        let subscribers = Vec::new();

        let (publisher, publisher_recv) = mpsc::channel(1);
        Ok((
            Self {
                queue_backend,
                subscribers,
                publisher_recv,
            },
            publisher,
        ))
    }
    pub async fn subscribe(&mut self, subscriber: mpsc::Sender<QueueEvent>) -> Result<()> {
        self.subscribers.push(subscriber);
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.queue_backend.recv().await {
            let futures = self
                .subscribers
                .iter()
                .map(|subscriber| subscriber.send(QueueEvent::Message(Message::new("".into()))))
                .collect::<Vec<_>>();

            futures::future::join_all(futures).await;
        }
        while let Some(message) = self.publisher_recv.recv().await {
            self.queue_backend
                .send(BackendMessage {
                    topic: "topic".into(),
                    payload: Vec::new(),
                })
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        queue_backend::{BackendMessage, MockQueueBackend},
        *,
    };

    use mockall::{predicate::*, Sequence};

    #[tokio::test]
    async fn establish_connection_with_queue() {
        let expected_subscribe_topic = "topic";

        let mut seq = Sequence::new();
        let mut queue_backend_mock = MockQueueBackend::new();
        queue_backend_mock
            .expect_connect()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));
        queue_backend_mock
            .expect_subscribe()
            .times(1)
            .with(eq(expected_subscribe_topic))
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        let tested_queue = Queue::new(queue_backend_mock).await;
        assert!(tested_queue.is_ok());
    }

    #[tokio::test]
    async fn forward_queue_messages_to_subscribers() {
        let expected_event = QueueEvent::Message(Message::new("".into()));

        let expected_subscribe_topic = "topic";
        let mut returned_messages = vec![BackendMessage {
            topic: expected_subscribe_topic.to_string(),
            payload: vec![],
        }]
        .into_iter();

        let mut seq = Sequence::new();
        let mut queue_backend_mock = MockQueueBackend::new();
        queue_backend_mock
            .expect_connect()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));
        queue_backend_mock
            .expect_subscribe()
            .times(1)
            .with(eq(expected_subscribe_topic))
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        queue_backend_mock
            .expect_recv()
            .times(2)
            .in_sequence(&mut seq)
            .returning(move || returned_messages.next());

        let (mut tested_queue, _) = Queue::new(queue_backend_mock).await.unwrap();
        let (sender1, mut receiver1) = mpsc::channel(1);
        let (sender2, mut receiver2) = mpsc::channel(1);

        tested_queue.subscribe(sender1).await.unwrap();
        tested_queue.subscribe(sender2).await.unwrap();

        let tested = tested_queue.run();

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

    #[tokio::test]
    async fn forward_messages_from_publishers_to_queue() {
        let expected_message = BackendMessage {
            topic: "topic".into(),
            payload: Vec::new(),
        };
        let event_to_publish = QueueEvent::Message(Message::new("".into()));

        let expected_subscribe_topic = "topic";
        let mut messages_from_queue = vec![].into_iter();

        let mut seq = Sequence::new();
        let mut queue_backend_mock = MockQueueBackend::new();
        queue_backend_mock
            .expect_connect()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));
        queue_backend_mock
            .expect_subscribe()
            .times(1)
            .with(eq(expected_subscribe_topic))
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        queue_backend_mock
            .expect_recv()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move || messages_from_queue.next());

        queue_backend_mock
            .expect_send()
            .times(1)
            .with(eq(expected_message))
            .returning(move |_| Ok(()));

        let (mut tested_queue, publisher) = Queue::new(queue_backend_mock).await.unwrap();

        let send = async move { publisher.send(event_to_publish).await };

        let tested = tested_queue.run();

        let r = tokio::try_join!(
            tokio::time::timeout(std::time::Duration::from_millis(100), tested),
            tokio::time::timeout(std::time::Duration::from_millis(100), send)
        )
        .unwrap();

        assert!(r.0.is_ok());
        assert!(r.1.is_ok());
    }
}
