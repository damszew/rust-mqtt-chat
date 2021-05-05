pub mod app;
pub mod events_handler;
pub mod events_reader;
pub mod network;
pub mod renderer;

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    // TODO: Rename to TerminalEvents and move to event_handler
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
