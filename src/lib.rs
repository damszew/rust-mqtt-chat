pub mod app;
pub mod events_handler;
pub mod renderer;

#[derive(Debug, PartialEq)]
pub enum AppEvent {
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
