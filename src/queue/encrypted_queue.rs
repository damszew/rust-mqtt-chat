use super::{Error, Message, Queue};
use crate::crypt::{Decrypt, Encrypt};

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
    C: Encrypt + Decrypt + Send + Sync,
{
    async fn publish(&self, topic: String, message: Message) -> Result<(), Error> {
        let encrypted_msg = self.crypto.encrypt(message);
        self.queue.publish(topic, encrypted_msg).await
    }

    async fn subscribe(&mut self, topic: String) -> Result<(), Error> {
        self.queue.subscribe(topic).await
    }

    async fn receive(&mut self) -> Result<Message, Error> {
        let encrypted_msg = self.queue.receive().await?;
        self.crypto.decrypt(encrypted_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::crypt::MockCrypto;
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
    async fn should_decrypt_received_message() {
        let mut crypto_mock = MockCrypto::new();
        crypto_mock.expect_decrypt().times(1).returning(Ok);

        let mut queue_mock = MockQueue::new();
        queue_mock
            .expect_receive()
            .times(1)
            .returning(|| Ok("test_data".as_bytes().to_owned()));

        let mut sut = EncryptedQueue::new(queue_mock, crypto_mock);

        let result = sut.receive().await.unwrap();

        assert_eq!(result, "test_data".as_bytes().to_owned());
    }
}
