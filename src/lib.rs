pub mod app;
pub mod crypt;
pub mod events_publisher;
pub mod events_reader;
pub mod network;
pub mod renderer;

pub mod queue;

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
