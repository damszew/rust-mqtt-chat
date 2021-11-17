use std::sync::{Arc, Mutex};

use super::{ChatMessage, ChatRoom, Error};
use crate::queue::Queue;

type SubscriberCallback = dyn Fn(ChatMessage) + Send + Sync + 'static;

const TOPIC_PREFIX: &str = "df9ff5c8-c030-4e4a-8bae-a415565febd7";

#[derive(Clone)]
pub struct QueueChatRoom<Q> {
    queue: Q,
    topic: String,
    user_name: String,
    subscribers: Arc<Mutex<Vec<Box<SubscriberCallback>>>>,
}

impl<Q> QueueChatRoom<Q>
where
    Q: Queue,
{
    pub async fn new(mut queue: Q, user_name: String, room_name: String) -> Result<Self, Error> {
        let topic = format!("{}/{}", TOPIC_PREFIX, room_name); // TODO: Remove tight coupling with mqtt topic format
        queue.subscribe(format!("{}/#", topic)).await?;

        let topic = format!("{}/{}", topic, user_name);
        Ok(Self {
            queue,
            topic,
            user_name,
            subscribers: Arc::default(),
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Ok(msg) = self.queue.receive().await {
            let msg = serde_json::from_slice(&msg)?;
            self.notify_subscribers(msg);
        }

        Ok(())
    }

    fn notify_subscribers(&self, msg: ChatMessage) {
        self.subscribers
            .lock()
            .expect("Poisoned mutex")
            .iter()
            .for_each(|subscriber| subscriber(msg.clone()));
    }
}

#[async_trait::async_trait]
impl<Q> ChatRoom for QueueChatRoom<Q>
where
    Q: Queue + Sync + Send,
{
    async fn send(&self, msg: String) -> Result<(), Error> {
        let msg = ChatMessage {
            user: self.user_name.clone(),
            msg,
            time: chrono::Local::now(),
        };
        let msg = serde_json::to_vec(&msg)?;

        self.queue.publish(self.topic.clone(), msg).await
    }

    fn on_message<F>(&mut self, callback: F)
    where
        F: Fn(ChatMessage) + Send + Sync + 'static,
    {
        self.subscribers
            .lock()
            .expect("Poisoned mutex")
            .push(Box::new(callback));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::queue::MockQueue;

    #[tokio::test]
    async fn should_subscribe_to_queue() {
        let mut queue_mock = MockQueue::new();
        queue_mock.expect_subscribe().times(1).returning(|_| Ok(()));

        let sut = QueueChatRoom::new(queue_mock, "user".to_string(), "room".to_string()).await;

        assert!(sut.is_ok());
    }

    #[tokio::test]
    async fn should_publish_message_to_queue() {
        let mut queue_mock = MockQueue::new();
        queue_mock.expect_subscribe().times(1).returning(|_| Ok(()));
        queue_mock
            .expect_publish()
            .withf(|_, msg| {
                let msg = serde_json::from_slice::<ChatMessage>(msg).unwrap();
                msg.user == "user" && msg.msg == "text message"
            })
            .times(1..)
            .returning(|_, _| Ok(()));

        let sut = QueueChatRoom::new(queue_mock, "user".to_string(), "room".to_string())
            .await
            .unwrap();
        let result = sut.send("text message".to_string()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_publish_message_to_correct_topic() {
        let mut queue_mock = MockQueue::new();
        queue_mock.expect_subscribe().times(1).returning(|_| Ok(()));
        queue_mock
            .expect_publish()
            .withf(|topic, _| topic.contains("room/user"))
            .times(1..)
            .returning(|_, _| Ok(()));

        let sut = QueueChatRoom::new(queue_mock, "user".to_string(), "room".to_string())
            .await
            .unwrap();
        let result = sut.send("text message".to_string()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_nofity_when_message_is_received_from_queue() {
        let mut queue_mock = MockQueue::new();
        queue_mock.expect_subscribe().times(1).returning(|_| Ok(()));
        queue_mock.expect_receive().times(1).returning(|| {
            Ok(serde_json::to_vec(&ChatMessage {
                user: "user".into(),
                msg: "text".into(),
                time: chrono::Local::now(),
            })
            .unwrap())
        });
        queue_mock
            .expect_receive()
            .times(1)
            .returning(|| Err(anyhow::anyhow!("finished")));

        let mut sut = QueueChatRoom::new(queue_mock, "user".to_string(), "room".to_string())
            .await
            .unwrap();

        let msg_received = std::sync::Arc::new(std::sync::Mutex::new(false));
        sut.on_message({
            let msg_received = msg_received.clone();
            move |_| {
                *msg_received.lock().unwrap() = true;
            }
        });

        let _ = tokio::time::timeout(std::time::Duration::from_millis(10), sut.run()).await;

        assert!(*msg_received.lock().unwrap());
    }
}
