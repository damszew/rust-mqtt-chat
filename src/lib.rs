pub mod app;
pub mod crypto;
pub mod events_publisher;
pub mod events_reader;
pub mod network;
pub mod renderer;

pub mod chat_room;
pub mod queue;
pub mod terminal_driver;
pub mod ui;

#[derive(Clone, Debug, PartialEq)]
pub enum TerminalEvent {
    Quit,
    Character(char),
    Accept,
    Remove,
    RemoveLast,
    CursorLeft,
    CursorRight,
    CursorStart,
    CursorEnd,
    ScrollUp,
    ScrollDown,
}
