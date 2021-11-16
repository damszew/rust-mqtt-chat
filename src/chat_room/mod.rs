pub mod queue_chat_room;

type Error = anyhow::Error;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user: String,
    pub msg: String,
    pub time: DateTime<Local>,
}

#[async_trait::async_trait]
pub trait ChatRoom {
    fn get_messages(&self) -> Vec<ChatMessage>;

    async fn send(&self, msg: String) -> Result<(), Error>;
}
