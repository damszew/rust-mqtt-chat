use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user: String,
    pub msg: String,
    pub time: DateTime<Local>,
}
