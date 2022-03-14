use std::sync::{Arc, Mutex};

use super::{ChatMessage, ChatRoom, Error};
use crate::queue::Queue;

const TOPIC_PREFIX: &str = "df9ff5c8-c030-4e4a-8bae-a415565febd7";

#[derive(Clone)]
pub struct QueueChatRoom<Q> {
    queue: Q,
    topic: String,
    user_name: String,
    messages: Arc<Mutex<Vec<ChatMessage>>>,
}

impl<Q> QueueChatRoom<Q>
where
    Q: Queue,
{
    pub async fn new(mut queue: Q, user_name: String, room_name: String) -> Result<Self, Error> {
        let topic = format!("{}/{}", TOPIC_PREFIX, room_name); // TODO: Remove tight coupling with mqtt topic format
        queue.subscribe(format!("{}/#", topic)).await?;

        let messages = Arc::new(Mutex::new(Vec::new()));

        let topic = format!("{}/{}", topic, user_name);
        Ok(Self {
            queue,
            topic,
            user_name,
            messages,
        })
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        while let Ok(msg) = self.queue.receive().await {
            let msg = serde_json::from_slice(&msg)?;
            self.messages.lock().expect("Poisoned mutex").push(msg);
        }

        Ok(())
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

    fn get_messages(&self) -> Vec<ChatMessage> {
        let messages = self.messages.lock().expect("Poisoned mutex");

        messages.to_owned()
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
    async fn should_return_received_messages() {
        let time = chrono::Local::now();

        let mut queue_mock = MockQueue::new();
        queue_mock.expect_subscribe().times(1).returning(|_| Ok(()));
        queue_mock.expect_receive().times(1).returning(move || {
            Ok(serde_json::to_vec(&ChatMessage {
                user: "user".into(),
                msg: "text".into(),
                time,
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

        let _ = tokio::time::timeout(std::time::Duration::from_millis(10), sut.run()).await;

        let messages = sut.get_messages();
        assert_eq!(
            messages,
            vec![ChatMessage {
                user: "user".into(),
                time,
                msg: "text".into(),
            }]
        );
    }
}
