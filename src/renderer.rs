mod ui;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub mod terminal_renderer;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct State {
    pub quit: bool,
    pub current_user: String,
    pub input_message: String,
    pub cursor: usize,
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub user: String,
    pub msg: String,
    pub time: DateTime<Local>,
}

impl Message {
    pub fn new(user: String, msg: String) -> Self {
        Self {
            user,
            msg,
            time: Local::now(),
        }
    }
}
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.time.format("%H:%M:%S ").to_string() == other.time.format("%H:%M:%S ").to_string()
            && self.msg == other.msg
            && self.user == other.user
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            time: Local::now(),
            ..Default::default()
        }
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait Renderer {
    fn render(&mut self, state: &State) -> Result<()>;
}
