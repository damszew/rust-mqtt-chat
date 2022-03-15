use std::pin::Pin;

use futures::{Stream, StreamExt};

use super::{Error, Message, Queue};
use crate::crypto::{Decrypt, Encrypt};

#[derive(Clone)]
pub struct EncryptedQueue<Q, C> {
    queue: Q,
    crypto: C,
}

impl<Q, C> EncryptedQueue<Q, C>
where
    Q: Queue,
    C: Encrypt + Decrypt,
{
    pub fn new(queue: Q, crypto: C) -> Self {
        Self { queue, crypto }
    }
}

#[async_trait::async_trait]
impl<Q, C> Queue for EncryptedQueue<Q, C>
where
    Q: Queue + Send + Sync,
    C: Encrypt + Decrypt + Clone + Send + Sync + 'static,
{
    async fn publish(&self, topic: String, message: Message) -> Result<(), Error> {
        let encrypted_msg = self.crypto.encrypt(message);
        self.queue.publish(topic, encrypted_msg).await
    }

    async fn subscribe(&mut self, topic: String) -> Result<(), Error> {
        self.queue.subscribe(topic).await
    }

    fn stream(&mut self) -> Pin<Box<dyn Stream<Item = Result<Message, Error>>>> {
        let stream = self.queue.stream().map({
            let crypto = self.crypto.clone();
            move |msg| crypto.decrypt(msg?)
        });

        Box::pin(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::crypto::MockCrypto;
    use crate::queue::MockQueue;

    #[tokio::test]
    async fn should_encrypt_published_message() {
        let mut crypto_mock = MockCrypto::new();
        crypto_mock.expect_encrypt().times(1).returning(|d| d);

        let mut queue_mock = MockQueue::new();
        queue_mock
            .expect_publish()
            .times(1)
            .returning(|_, _| Ok(()));

        let sut = EncryptedQueue::new(queue_mock, crypto_mock);

        let result = sut
            .publish("test_topic".to_string(), "some data".as_bytes().to_owned())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_decrypt_messages_received_from_stream() {
        let mut crypto_mock = MockCrypto::new();

        crypto_mock.expect_clone().returning(|| {
            let mut mock = MockCrypto::new();
            mock.expect_decrypt().returning(Ok);
            mock
        });

        let mut queue_mock = MockQueue::new();
        queue_mock.expect_stream().returning(|| {
            futures::stream::once(async { Ok("test_data".as_bytes().to_owned()) }).boxed()
        });

        let mut sut = EncryptedQueue::new(queue_mock, crypto_mock);

        let result = sut.stream().map(Result::unwrap).collect::<Vec<_>>().await;

        assert_eq!(result, vec!["test_data".as_bytes().to_owned()]);
    }
}
