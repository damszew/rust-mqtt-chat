mod ui;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

pub mod terminal_renderer;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct State {
    pub input_message: String,
    pub cursor: usize,
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub time: DateTime<Local>,
    pub msg: String,
}

impl Message {
    pub fn new(msg: String) -> Self {
        Self {
            time: Local::now(),
            msg,
        }
    }
}
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.time.format("%H:%M:%S ").to_string() == other.time.format("%H:%M:%S ").to_string()
            && self.msg == other.msg
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
