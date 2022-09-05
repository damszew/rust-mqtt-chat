pub mod queue_chat_room;

type Error = anyhow::Error;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user: String,
    pub msg: String,
    pub time: DateTime<Local>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ChatRoom {
    async fn send(&self, msg: String) -> Result<(), Error>;
    fn get_messages(&self) -> Vec<ChatMessage>;
}
