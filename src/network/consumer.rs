use anyhow::Result;
use futures::StreamExt;
use paho_mqtt::Message;
use tokio::sync::mpsc;

use super::NetworkEvent;

pub struct MqttConsumer {
    pub mqtt_receiver: futures::channel::mpsc::Receiver<Option<Message>>,
    pub subscribers: Vec<mpsc::Sender<NetworkEvent>>,
}

impl MqttConsumer {
    pub async fn subscribe(&mut self, subscriber: mpsc::Sender<NetworkEvent>) -> Result<()> {
        self.subscribers.push(subscriber);
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(Some(message)) = self.mqtt_receiver.next().await {
            let message = message.payload().to_owned();

            let futures = self
                .subscribers
                .iter()
                .map(|subscriber| subscriber.send(NetworkEvent::Message(message.clone())))
                .collect::<Vec<_>>();

            futures::future::join_all(futures).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_case::test_case;

    #[test_case(vec![
        "".as_bytes().to_owned(),
    ] => vec![
        NetworkEvent::Message("".into()),
    ] ; "Handle empty payload")]
    #[test_case(vec![
        "payload 1".as_bytes().to_owned(),
        "payload 2".as_bytes().to_owned(),
    ] => vec![
        NetworkEvent::Message("payload 1".into()),
        NetworkEvent::Message("payload 2".into()),
    ] ; "Handle multiple different messages")]
    #[test_case(vec![
        "the same payload".as_bytes().to_owned(),
        "the same payload".as_bytes().to_owned(),
    ] => vec![
        NetworkEvent::Message("the same payload".into()),
        NetworkEvent::Message("the same payload".into()),
    ] ; "Handle multiple similar messages")]
    #[test_case(vec![
        "녹".as_bytes().to_owned(),
        "😎".as_bytes().to_owned(),
    ] => vec![
        NetworkEvent::Message("녹".into()),
        NetworkEvent::Message("😎".into()),
    ] ; "Handle non ascii messages")]
    #[tokio::test]
    async fn receive_couple_messages(network_messages: Vec<Vec<u8>>) -> Vec<NetworkEvent> {
        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let (test_subscriber, mut test_receiver) = mpsc::channel(1);

        let mut tested_consumer = MqttConsumer {
            mqtt_receiver: network_backend_mock,
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
        let expected_event = NetworkEvent::Message(payload.into());
        let network_messages = vec![payload.as_bytes().to_owned()];

        let network_backend_mock = setup_network_mock(network_messages.into_iter());

        let (test_subscriber1, mut test_receiver1) = mpsc::channel(1);
        let (test_subscriber2, mut test_receiver2) = mpsc::channel(1);

        let mut tested_consumer = MqttConsumer {
            mqtt_receiver: network_backend_mock,
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

    fn setup_network_mock(
        network_messages: std::vec::IntoIter<Vec<u8>>,
    ) -> futures::channel::mpsc::Receiver<Option<Message>> {
        let (mut sender, receiver) = futures::channel::mpsc::channel(network_messages.len() + 1);
        for m in network_messages {
            let mqtt_message = Message::new("topic", m, 0);
            sender.try_send(Some(mqtt_message)).unwrap();
        }
        sender.try_send(None).unwrap();

        receiver
    }
}
